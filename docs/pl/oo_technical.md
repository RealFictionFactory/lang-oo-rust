# Język "Ó" - Dokumentacja Techniczna

Ten dokument opisuje architekturę interpretera języka "Ó" napisanego w języku Rust.

## 1. Architektura
Interpreter implementuje klasyczny potok tłumaczenia i wykonania kodu:
`Kod źródłowy (String)` -> **Lekser** -> `Tokeny` -> **Parser** -> `AST` -> **Interpreter** -> `Wykonanie`

Program działa w dwóch trybach:
*   **REPL:** Interaktywna konsola uruchamiana poleceniem `cargo run`. Znak zachęty to `ó>`. Kod wykonuje się po wpisaniu `.` w pustej linii.
*   **Tryb plikowy:** Uruchamiany przez `cargo run -- plik.oo`. Czyta i wykonuje cały plik. Obsługuje uniksowe shebangi (`#!`) do natywnego skryptowania w shellu. Zwraca kod wyjścia `0` w przypadku sukcesu i `1` w przypadku błędów wykonania/składni.

REPL utrzymuje jedno `Environment` przez cały czas sesji, więc powiązania zmiennych są zachowywane między kolejnymi wejściami, a błąd składni lub wykonania jest zgłaszany bez kończenia sesji. Tryb plikowy tworzy własne środowisko i kończy się kodem różnym od zera, gdy skrypt zawiedzie.

## 2. Moduły (Struktura Projektu)

Projekt jest podzielony na logiczne moduły w folderze `src/`:

### `lexer.rs`
- Odpowiada za analizę leksykalną.
- Zmienia ciąg znaków (`String`) na wektor tokenów (`Vec<Token>`).
- Rozpoznaje słowa kluczowe (w tym `fun`, `and`, `or`, `not`, `execute`, `onError`, `until`, `in`, `match`) i operatory (w tym `->`, `??`).
- Posiada logikę "podglądania" (peek) dla operatorów dwuznakowych (np. `==`, `+=`, `..`, `<=`, `->`, `??`).
- Automatycznie rozpoznaje ułamki, sprawdzając czy kropka po liczbie nie jest początkiem zakresu `..`.
- Ignoruje komentarze `//` aż do końca linii. Ignoruje również shebangi `#!` w pierwszej linii pliku.
- Nigdy nie panikuje na błędnym wejściu. Nieznany znak, samotny `!`/`?`, literał liczbowy poza zakresem lub niedomknięty literał tekstowy tworzą `Token::Error(String)` niosący komunikat; parser przedstawia go jako zwykły błąd składni. To właśnie sprawia, że błędy składni są odzyskiwalne, a REPL odporny na błędy.

### `ast.rs`
- Zawiera definicje struktury drzewa Abstract Syntax Tree (AST).
- `Expr`: Wyrażenia, które po obliczeniu zwracają wartość. Obejmuje operacje matematyczne, dostęp do zmiennych, wywołania funkcji, `If` (działające jak wyrażenie trójargumentowe), `ExecuteCatch` (obsługa błędów), `Unary` (negacja liczby/instrukcja `not`), `Dict` (literały słowników), `Match` (wzorce dopasowania), `NullCoalesce` (operator `??`) i `Lambda` (funkcje anonimowe).
- `Stmt`: Instrukcje, które robią coś w kodzie (deklaracje, pętle, przypisania), ale nie zwracają wartości. Obejmuje `LoopIn` (iteracja po tablicy), `LoopBlock` (pętla nieskończona) i `Until` (warunkowe przerwanie pętli).
- Drzewa rekurencyjne są opakowane w `Box<T>` z racji wymogów Rusta dot. rozmiaru struktur.
- Lista parametrów i ciało funkcji (`FuncDecl`, `Lambda`) są przechowywane za `Rc`, dzięki czemu utworzenie wartości wywoływalnej klonuje wskaźnik, zamiast głęboko kopiować całe ciało przy każdym odczycie.

### `parser.rs`
- Implementuje technikę *Recursive Descent Parsing*.
- Buduje AST na podstawie `Vec<Token>` zwróconego przez lekser.
- Rozdziela parsowanie wyrażeń na jedną funkcję na każdy poziom priorytetu, od najsłabszego do najsilniejszego: `parse_expr` (`??`) -> `parse_or` -> `parse_and` -> `parse_equality` (`==`, `!=`) -> `parse_comparison` (`<`, `>`, `<=`, `>=`) -> `parse_additive` (`+`, `-`) -> `parse_term` (`*`, `/`, `%`) -> `parse_unary` (`-`, `not`) -> `parse_factor` (wartości, nawiasy). Każdy poziom deleguje do silniej wiążącego, więc `and` wiąże silniej niż `or`, a porównania słabiej niż arytmetyka.
- Używa mechanizmów desugaringu: np. `+=` zamienia na drzewo przypisania z dodawaniem w locie, a interpolacja stringów `"a {b}"` tworzy wewnętrzną instancję Leksera i Parsera, by zwrócić wyrażenie `Expr::Binary("a " + b)`.
- Kontekstowe parsowanie w `parse_factor`: jeśli napotka `{` w miejscu, gdzie oczekuje wartości, traktuje to jako literał `Dict`; jeśli w miejscu instrukcji – jako blok kodu.
- `if`, `execute` i `match` są parsowane jako wyrażenia, co pozwala na ich użycie inline (np. `var x = if ...`).
- Wywołania funkcji i metod są parsowane "postfiksowo" na końcu `parse_factor` w jednej pętli, co pozwala na nieograniczone łańcuchowanie (np. `foo()().bar()[0]`).

### `interpreter.rs`
- Implementuje logikę wykonania na drzewie AST.
- Główna architektura opiera się na `Rc<RefCell<T>>` dla współdzielonej mutowalności i wydajności pamięci:
  - `Environment` jest opakowane w `Rc<RefCell<Environment>>`. Implementuje wzorzec łańcucha zasięgów (Scope Chain) z polem `parent: Option<Rc<RefCell<Environment>>>`, co pozwala funkcjom i domknięciom współdzielić stan i czytać zmienne globalne bez klonowania całego środowiska. Każdy blok (`if`, `match`, `execute`/`onError` oraz wszystkie trzy formy pętli) działa we własnym zasięgu potomnym, więc deklaracja wewnątrz bloku, zmienna iteratora pętli czy zmienna `onError` nie wyciekają do zewnętrznego zasięgu ani go nie nadpisują. Zasięgi potomne są tanie, bo rejestr rozszerzeń jest rozwiązywany przez łańcuch rodziców (`find_extension`), a nie kopiowany dla każdego zasięgu.
  - `Value::Array` (`Rc<RefCell<Vec<Value>>>`, `Rc<Cell<bool>>`) oraz `Value::Dict` (`Rc<RefCell<HashMap<String, Value>>>`, `Rc<Cell<bool>>`) to kontenery. Każdy niesie **flagę niemutowalności** podróżującą wraz z obiektem. `push` i przypisanie indeksowane sprawdzają tę flagę, więc kontener `let` jest tylko do odczytu, niezależnie od tego, jak się do niego dostajemy.
- **Wartości funkcyjne przechwytują swój zakres definiujący słabo, a funkcje są przejściowe.** `Value::Function` przechowuje `Weak<RefCell<Environment>>` zakresu, w którym je napisano (`Rc::downgrade(env)` przy `Stmt::FuncDecl`/`Expr::Lambda`), a nie silne `Rc`. Słaba referencja nigdy nie utrzymuje środowiska przy życiu, więc cykl `środowisko → funkcja → środowisko`, którego zliczanie referencji nie potrafi zebrać, nie może powstać — porzucenie ostatniego zewnętrznego uchwytu do środowiska zwalnia je (istnieje test regresyjny, który degraduje środowisko do `Weak`, porzuca je i sprawdza, że `Weak` już się nie „podnosi"). `execute_function` podnosi (upgrade) słabą referencję przy wywołaniu funkcji; w poprawnym programie funkcja jest callbackiem wciąż obecnym na stosie wywołań, więc jej zakres żyje, a jeśli podniesienie kiedyś zawiedzie, zwracany jest czysty błąd wykonania zamiast niepoprawnego działania. Aby słabe przechwycenie było bezpieczne, funkcje są ograniczone do **przejściowych, synchronicznych callbacków**: `Stmt::FuncDecl` jest dozwolony tylko w zakresie globalnym, a każda próba przechowania wartości funkcyjnej — przypisanie, powiązanie `let`/`var`, `return`, literał tablicy lub słownika, zapis indeksowany albo `push` — jest odrzucana (`reject_stored_function`). Funkcję można nadal przekazać bezpośrednio jako argument wywołania i może ona widzieć zmienne lokalne swojego zakresu leksykalnego w trakcie działania.
- **Semantyka wartości dla kontenerów.** `deep_bind(value, immutable)` daje każdemu powiązaniu własny kontener: kontener osiągnięty przez alias jest głęboko kopiowany, a każdy poziom oznaczany mutowalnością powiązania (`let` → niemutowalny, `var` → mutowalny), natomiast wartość będąca wyłącznym właścicielem (świeży literał lub wartość zwrócona z funkcji) jest oznaczana w miejscu, aby uniknąć zbędnej kopii. Jest wywoływana przy `Stmt::VarDecl`, `Stmt::Let`, `Stmt::Assign` oraz przy zapisie do gniazda kontenera (przypisanie indeksowane, `push`). Efekt: dwie nazwane zmienne nigdy nie współdzielą jednego obiektu. **Parametry funkcji są zamierzonym wyjątkiem**: `execute_function` klonuje `Rc` argumentu (współdzieli przez referencję), więc niemutowalność podróżuje do funkcji, a mutowalny kontener może być modyfikowany w miejscu — jedyna zamierzona ścieżka aliasowania.
- Struktura `VarInfo` przechowuje `Value`, flagę `is_const` (czy *nazwę* można ponownie związać) oraz opcjonalne `type_name`. To umożliwia **sprawdzanie typów w czasie wykonania (Runtime Type Checking)**: jeśli określono `type_name`, interpreter waliduje wartości podczas deklaracji (`Stmt::VarDecl`, `Stmt::Let`) i przypisania (`Stmt::Assign`). Akceptowane są tylko znane nazwy typów; nieznana adnotacja jest odrzucana od razu (`is_known_type`). Zwróć uwagę, że *mutowalność* kontenera jest cechą obiektu (flaga niemutowalności), odrębną od `is_const`, które rządzi jedynie ponownym wiązaniem nazwy.
- Rdzeń interpretera jest "głupi" i agnostyczny wobec konkretnych typów. Nie posiada żadnych na sztywno wpisanych metod (jak `push` czy `read`). Rozumie tylko ewaluację AST, zarządzanie zasięgiem (scope) i przepływ sterowania.
- Arytmetyka liczb całkowitych używa operacji z kontrolą (`checked_add`, `checked_div`, `checked_neg`, …): przepełnienie staje się błędem wykonania `InterpErr::Err`, zamiast panikować w kompilacjach debug lub po cichu się zawijać w kompilacjach release.
- Wykorzystuje własny system błędów `InterpErr` z wariantami `Return`, `Break`, `Continue` i `Err`. Dzięki propagacji błędów (`?`), instrukcje sterujące "przebijają się" przez zagnieżdżone bloki. Instrukcja `Until` wykorzystuje to, zwracając `InterpErr::Break`, jeśli jej warunek jest prawdziwy.
- Wprowadza `eval_block_as_expr`, funkcję pomocniczą, która ewaluuje blok instrukcji (w świeżym zasięgu potomnym) i zwraca wartość ostatniego wyrażenia, co pozwala `if`, `execute` i `match` działać jako wyrażenia.
- `execute_function` to publiczna metoda pomocnicza, która wykonuje `Value::Function` lub `Value::Builtin` z podanymi argumentami. Pozwala to bibliotece standardowej (np. `.map()`, `.filter()`) na uruchamianie lambd/domknięć przekazanych z poziomu języka "Ó".
- `ExecuteCatch` przechwytuje `InterpErr::Err` (przypisując komunikat do zmiennej), pozwalając jednocześnie błędom przepływu sterowania (`Return`, `Break`, `Continue`) na naturalną propagację.

### `stdlib.rs`
- Pełni rolę biblioteki standardowej języka.
- Rejestruje globalne funkcje wbudowane (`print`, `input`, `args`, `exit`, `shell`) oraz czyste metody rozszerzające (pure extension methods) w środowisku `Environment`.
- Implementuje metody rozszerzające dla Stringów (`upper`, `lower`, `trim`, `contains`, `replace`, `split`) i Tablic (`contains`, `join`, `push`, `map`, `filter`). Metody tablicowe takie jak `map` czy `filter` wykorzystują helper `Environment::execute_function` do uruchamiania przekazanych lambd.
- Metody rozszerzające walidują liczbę argumentów za pomocą `check_arity` i zwracają błąd wykonania zamiast panikować, gdy wywołano je ze zbyt małą liczbą argumentów. `push` dodatkowo sprawdza flagę niemutowalności odbiorcy (odrzucając tablicę `let`) i zapisuje niezależną kopię elementu przez `Environment::deep_bind`.
- Trzymanie tej logiki z dala od `interpreter.rs` sprawia, że rdzeń interpretera pozostaje lekki i ogólny.

### `modules/`
- Obsługuje zewnętrzne rozszerzenia lub biblioteki standardowe ładowane przez słowo kluczowe `use`.
- `modules/io.rs`: Implementuje obiekt `File` przy użyciu słownika pod maską (przechowującego klucze `path` i `__type__`). Rejestruje globalny konstruktor `file(path)` oraz metody rozszerzające (`read`, `write`, `append`, `exists`). Ten design sprawia, że interpreter jest całkowicie agnostyczny wobec operacji na plikach.

### `main.rs`
- Punkt wejścia programu. Odpowiada za CLI, czyta wejście z konsoli lub pliku i wywołuje moduły w odpowiedniej kolejności: Lexer -> Parser -> Interpreter.
- `run_code` przyjmuje `Environment` jako parametr i zwraca `Result<(), ()>` zamiast kończyć proces, dzięki czemu REPL może ponownie użyć jednego środowiska między wejściami i działać dalej po błędzie. Tryb plikowy tworzy własne środowisko i mapuje niepowodzenie na kod wyjścia `1` (sukces `0`).

### `tests.rs`
- Moduł testów jednostkowych. Używa atrybutu `#[cfg(test)]`.
- Przeprowadza testy integracyjne całego potoku: Lexer -> Parser -> Interpreter, weryfikując wyniki, struktury AST i poprawne wyrzucanie błędów (w tym operatory logiczne, priorytety operatorów, pętle, słowniki, bezpieczny dostęp do słowników, obsługę błędów, pattern matching, domknięcia, operacje na plikach, sprawdzanie typów, zasięgi bloków, przepełnienie liczb całkowitych oraz semantykę wartości / niemutowalność kontenerów). (Obecnie 99 przechodzących testów).

## 3. Zależności
Projekt w pełni opiera się na bibliotece standardowej Rusta (`std`). Nie używa żadnych zewnętrznych skrzynek (Crate'ów).
- `std::collections::HashMap` (dla środowisk, tablic i słowników).
- `std::io` i `std::fs` (do interakcji z CLI, operacjach na plikach i module I/O).
- `std::env` (do czytania argumentów startowych).
- `std::process` (do wykonywania komend systemowych i obsługi kodów wyjścia).
- `std::cell::RefCell`, `std::cell::Cell` i `std::rc::Rc` (dla współdzielonej mutowalności, flagi niemutowalności kontenerów i zarządzania pamięcią).
