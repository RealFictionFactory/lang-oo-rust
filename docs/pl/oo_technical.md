# Język "Ó" - Dokumentacja Techniczna

Ten dokument opisuje architekturę interpretera języka "Ó" napisanego w języku Rust.

## 1. Architektura
Interpreter implementuje klasyczny potok tłumaczenia i wykonania kodu:
`Kod źródłowy (String)` -> **Lekser** -> `Tokeny` -> **Parser** -> `AST` -> **Interpreter** -> `Wykonanie`

Program działa w dwóch trybach:
*   **REPL:** Interaktywna konsola uruchamiana poleceniem `cargo run`. Znak zachęty to `ó>`. Kod wykonuje się po wpisaniu `.` w pustej linii.
*   **Tryb plikowy:** Uruchamiany przez `cargo run -- plik.oo`. Czyta i wykonuje cały plik. Obsługuje uniksowe shebangi (`#!`) do natywnego skryptowania w shellu. Zwraca kod wyjścia `0` w przypadku sukcesu i `1` w przypadku błędów wykonania/składni.

## 2. Moduły (Struktura Projektu)

Projekt jest podzielony na logiczne moduły w folderze `src/`:

### `lexer.rs`
- Odpowiada za analizę leksykalną.
- Zmienia ciąg znaków (`String`) na wektor tokenów (`Vec<Token>`).
- Rozpoznaje słowa kluczowe (w tym `fun`, `and`, `or`, `not`, `execute`, `onError`, `until`, `in`, `match`) i operatory (w tym `->`, `??`).
- Posiada logikę "podglądania" (peek) dla operatorów dwuznakowych (np. `==`, `+=`, `..`, `<=`, `->`, `??`).
- Automatycznie rozpoznaje ułamki, sprawdzając czy kropka po liczbie nie jest początkiem zakresu `..`.
- Ignoruje komentarze `//` aż do końca linii. Ignoruje również shebangi `#!` w pierwszej linii pliku.

### `ast.rs`
- Zawiera definicje struktury drzewa Abstract Syntax Tree (AST).
- `Expr`: Wyrażenia, które po obliczeniu zwracają wartość. Obejmuje operacje matematyczne, dostęp do zmiennych, wywołania funkcji, `If` (działające jak wyrażenie trójargumentowe), `ExecuteCatch` (obsługa błędów), `Unary` (negacja liczby/instrukcja `not`), `Dict` (literały słowników), `Match` (wzorce dopasowania), `NullCoalesce` (operator `??`) i `Lambda` (funkcje anonimowe).
- `Stmt`: Instrukcje, które robią coś w kodzie (deklaracje, pętle, przypisania), ale nie zwracają wartości. Obejmuje `LoopIn` (iteracja po tablicy), `LoopBlock` (pętla nieskończona) i `Until` (warunkowe przerwanie pętli).
- Drzewa rekurencyjne są opakowane w `Box<T>` z racji wymogów Rusta dot. rozmiaru struktur.

### `parser.rs`
- Implementuje technikę *Recursive Descent Parsing*.
- Buduje AST na podstawie `Vec<Token>` zwróconego przez lekser.
- Rozdziela parsowanie wyrażeń na ścisłe poziomy priorytetów: `parse_expr` (and, or, `??`) -> `parse_logic` (+, -, porównania) -> `parse_term` (*, /, %) -> `parse_unary` (-, not) -> `parse_factor` (wartości, nawiasy).
- Używa mechanizmów desugaringu: np. `+=` zamienia na drzewo przypisania z dodawaniem w locie, a interpolacja stringów `"a {b}"` tworzy wewnętrzną instancję Leksera i Parsera, by zwrócić wyrażenie `Expr::Binary("a " + b)`.
- Kontekstowe parsowanie w `parse_factor`: jeśli napotka `{` w miejscu, gdzie oczekuje wartości, traktuje to jako literał `Dict`; jeśli w miejscu instrukcji – jako blok kodu.
- `if`, `execute` i `match` są parsowane jako wyrażenia, co pozwala na ich użycie inline (np. `var x = if ...`).
- Wywołania funkcji i metod są parsowane "postfiksowo" na końcu `parse_factor` w jednej pętli, co pozwala na nieograniczone łańcuchowanie (np. `foo()().bar()[0]`).

### `interpreter.rs`
- Implementuje logikę wykonania na drzewie AST.
- Główna architektura opiera się na `Rc<RefCell<T>>` dla współdzielonej mutowalności i wydajności pamięci:
  - `Environment` jest opakowane w `Rc<RefCell<Environment>>`. Implementuje wzorzec łańcucha zasięgów (Scope Chain) z polem `parent: Option<Rc<RefCell<Environment>>>`, co pozwala funkcjom i domknięciom współdzielić stan i czytać zmienne globalne bez klonowania całego środowiska.
  - `Value::Array` (`Rc<RefCell<Vec<Value>>>`) oraz `Value::Dict` (`Rc<RefCell<HashMap<String, Value>>>`) to typy referencyjne. Są przekazywane do funkcji przez referencję, co czyni je mutowalnymi wewnątrz funkcji bez kopiowania ich zawartości.
- Struktura `VarInfo` przechowuje `Value`, flagę `is_const` oraz opcjonalne `type_name`. To umożliwia **sprawdzanie typów w czasie wykonania (Runtime Type Checking)**: jeśli określono `type_name`, interpreter waliduje wartości podczas deklaracji (`Stmt::VarDecl`, `Stmt::Let`) i przypisania (`Stmt::Assign`).
- Rdzeń interpretera jest "głupi" i agnostyczny wobec konkretnych typów. Nie posiada żadnych na sztywno wpisanych metod (jak `push` czy `read`). Rozumie tylko ewaluację AST, zarządzanie zasięgiem (scope) i przepływ sterowania.
- Wykorzystuje własny system błędów `InterpErr` z wariantami `Return`, `Break`, `Continue` i `Err`. Dzięki propagacji błędów (`?`), instrukcje sterujące "przebijają się" przez zagnieżdżone bloki. Instrukcja `Until` wykorzystuje to, zwracając `InterpErr::Break`, jeśli jej warunek jest prawdziwy.
- Wprowadza `eval_block_as_expr`, funkcję pomocniczą, która ewaluuje blok instrukcji i zwraca wartość ostatniego wyrażenia, co pozwala `if`, `execute` i `match` działać jako wyrażenia.
- `execute_function` to publiczna metoda pomocnicza, która wykonuje `Value::Function` lub `Value::Builtin` z podanymi argumentami. Pozwala to bibliotece standardowej (np. `.map()`, `.filter()`) na uruchamianie lambd/domknięć przekazanych z poziomu języka "Ó".
- `ExecuteCatch` przechwytuje `InterpErr::Err` (przypisując komunikat do zmiennej), pozwalając jednocześnie błędom przepływu sterowania (`Return`, `Break`, `Continue`) na naturalną propagację.

### `stdlib.rs`
- Pełni rolę biblioteki standardowej języka.
- Rejestruje globalne funkcje wbudowane (`print`, `input`, `args`, `exit`, `shell`) oraz czyste metody rozszerzające (pure extension methods) w środowisku `Environment`.
- Implementuje metody rozszerzające dla Stringów (`upper`, `lower`, `trim`, `contains`, `replace`, `split`) i Tablic (`contains`, `join`, `push`, `map`, `filter`). Metody tablicowe takie jak `map` czy `filter` wykorzystują helper `Environment::execute_function` do uruchamiania przekazanych lambd.
- Trzymanie tej logiki z dala od `interpreter.rs` sprawia, że rdzeń interpretera pozostaje lekki i ogólny.

### `modules/`
- Obsługuje zewnętrzne rozszerzenia lub biblioteki standardowe ładowane przez słowo kluczowe `use`.
- `modules/io.rs`: Implementuje obiekt `File` przy użyciu słownika pod maską (przechowującego klucze `path` i `__type__`). Rejestruje globalny konstruktor `file(path)` oraz metody rozszerzające (`read`, `write`, `append`, `exists`). Ten design sprawia, że interpreter jest całkowicie agnostyczny wobec operacji na plikach.

### `main.rs`
- Punkt wejścia programu. Odpowiada za CLI, czyta wejście z konsoli lub pliku i wywołuje moduły w odpowiedniej kolejności: Lexer -> Parser -> Interpreter.
- Mapuje wynik działania interpretera na kody wyjścia procesu (`0` dla sukcesu, `1` dla błędów).

### `tests.rs`
- Moduł testów jednostkowych. Używa atrybutu `#[cfg(test)]`.
- Przeprowadza testy integracyjne całego potoku: Lexer -> Parser -> Interpreter, weryfikując wyniki, struktury AST i poprawne wyrzucanie błędów (w tym operatory logiczne, pętle, słowniki, bezpieczny dostęp do słowników, obsługę błędów, pattern matching, domknięcia, operacje na plikach i sprawdzanie typów). (Obecnie 68 przechodzących testów).

## 3. Zależności
Projekt w pełni opiera się na bibliotece standardowej Rusta (`std`). Nie używa żadnych zewnętrznych skrzynek (Crate'ów).
- `std::collections::HashMap` (dla środowisk, tablic i słowników).
- `std::io` i `std::fs` (do interakcji z CLI, operacjach na plikach i module I/O).
- `std::env` (do czytania argumentów startowych).
- `std::process` (do wykonywania komend systemowych i obsługi kodów wyjścia).
- `std::cell::RefCell` i `std::rc::Rc` (dla współdzielonej mutowalności i zarządzania pamięcią).
