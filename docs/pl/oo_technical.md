# Język "`Ó`" - Dokumentacja Techniczna

Ten dokument opisuje architekturę interpretera języka "`Ó`" napisanego w języku Rust.

## 1. Architektura
Interpreter implementuje klasyczny potok tłumaczenia i wykonania kodu:
`Kod źródłowy (String)` -> **Lekser** -> `Tokeny` -> **Parser** -> `AST` -> **Interpreter** -> `Wykonanie`

Program działa w dwóch trybach:
*   **REPL:** Interaktywna konsola uruchamiana poleceniem `cargo run`. Znak zachęty to `ó>`. Kod wykonuje się po wpisaniu `.` w pustej linii.
*   **Plikowy:** Uruchamiany przez `cargo run -- plik.oo`. Czyta i wykonuje cały plik.

## 2. Moduły (Struktura Projektu)

Projekt jest podzielony na logiczne moduły w folderze `src/`:

### `lexer.rs`
- Odpowiada za analizę leksykalną.
- Zmienia ciąg znaków (`String`) na wektor tokenów (`Vec<Token>`).
- Rozpoznaje słowa kluczowe (w tym `fun`, `and`, `or`, `not`, `execute`, `onError`, `until`, `in`), literały (Number, Decimal, String, Bool), identyfikatory i operatory.
- Posiada logikę "podglądania" (peek) dla operatorów dwuznakowych (np. `==`, `+=`, `..`, `<=`).
- Automatycznie rozpoznaje ułamki, sprawdzając czy kropka po liczbie nie jest początkiem zakresu `..`.
- Ignoruje komentarze `//` aż do końca linii, generując tokeny `NewLine` znaczące koniec instrukcji.

### `ast.rs`
- Zawiera definicje struktury drzewa Abstract Syntax Tree (AST).
- `Expr`: Wyrażenia, które po obliczeniu zwracają wartość. Obejmuje operacje matematyczne, dostęp do zmiennych, wywołania funkcji, `If` (działające jak wyrażenie trójargumentowe), `ExecuteCatch` (obsługa błędów), `Unary` (negacja liczby/instrukcja `not`) i `Dict` (literały słowników).
- `Stmt`: Instrukcje, które robią coś w kodzie (deklaracje, pętle, przypisania), ale nie zwracają wartości. Obejmuje `LoopIn` (iteracja po tablicy), `LoopBlock` (pętla nieskończona) i `Until` (warunkowe przerwanie pętli).
- Drzewa rekurencyjne są opakowane w `Box<T>` z racji wymogów Rusta dot. rozmiaru struktur.

### `parser.rs`
- Implementuje technikę *Recursive Descent Parsing*.
- Buduje AST na podstawie `Vec<Token>` zwróconego przez lekser.
- Rozdziela parsowanie wyrażeń na ścisłe poziomy priorytetów: `parse_expr` (and, or) -> `parse_logic` (+, -, porównania) -> `parse_term` (*, /, %) -> `parse_unary` (-, not) -> `parse_factor` (wartości, nawiasy).
- Używa mechanizmów desugaringu: np. `+=` zamienia na drzewo przypisania z dodawaniem w locie, a interpolacja stringów `"a {b}"` tworzy wewnętrzną instancję Leksera i Parsera, by zwrócić wyrażenie `Expr::Binary("a " + b)`.
- Kontekstowe parsowanie w `parse_factor`: jeśli napotka `{` w miejscu, gdzie oczekuje wartości, traktuje to jako literał `Dict`; jeśli w miejscu instrukcji – jako blok kodu.
- `if` oraz `execute` są parsowane jako wyrażenia, co pozwala na ich użycie inline (np. `var x = if ...`).
- Wywołania funkcji i metod są parsowane "postfiksowo" na końcu `parse_factor` w jednej pętli, co pozwala na nieograniczone łańcuchowanie (np. `foo()().bar()[0]`).

### `interpreter.rs`
- Implementuje logikę wykonania na drzewie AST.
- Główną strukturą jest `Environment`, implementująca wzorzec łańcucha zasięgów (Scope Chain) z polem `parent: Option<Box<Environment>>`. To pozwala funkcjom czytać zmienne globalne, nie nadpisując ich trwale swoimi lokalnymi.
- Struktura `VarInfo` przechowuje `Value`, flagę `is_const` oraz opcjonalne `type_name`. To umożliwia **sprawdzanie typów w czasie wykonania (Runtime Type Checking)**: jeśli określono `type_name`, interpreter waliduje wartości podczas deklaracji (`Stmt::VarDecl`, `Stmt::Let`) i przypisania (`Stmt::Assign`) używając metod pomocniczych takich jak `value_matches_type`.
- Wykorzystuje własny system błędów `InterpErr` z wariantami `Return`, `Break`, `Continue` i `Err`. Dzięki wykorzystaniu propagacji błędów (`?`), instrukcje sterujące "przebijają się" przez zagnieżdżone bloki. Instrukcja `Until` wykorzystuje to, zwracając `InterpErr::Break`, jeśli jej warunek jest prawdziwy.
- Wprowadza funkcję `eval_block_as_expr`, która ewaluuje blok instrukcji i zwraca wartość ostatniego wyrażenia, co pozwala `if` i `execute` działać jako wyrażenia.
- Implementuje short-circuit evaluation dla operatorów logicznych (`and`, `or`).
- `ExecuteCatch` przechwytuje `InterpErr::Err` (przypisując komunikat do zmiennej), pozwalając jednocześnie błędom przepływu sterowania (`Return`, `Break`, `Continue`) na naturalną propagację.
- Funkcje wbudowane (`print`, `input`) przechowywane są jako wskaźniki na funkcje Rustowe `BuiltinFn` wewnątrz enuma `Value`.
- Metody rozszerzające (np. `asNumber`, `upper`) przechowywane są w osobnej mapie `extensions` wewnątrz `Environment` i są wyszukiwane dynamicznie podczas ewaluacji `MethodCall`. Metody mutujące (jak `push` dla tablic lub przypisanie po indeksie dla słowników) są obsługiwane bezpośrednio w interpreterze, aby uzyskać dostęp do referencji środowiska.

### `stdlib.rs`
- Pełni rolę biblioteki standardowej języka.
- Rejestruje globalne funkcje wbudowane i metody rozszerzające w środowisku `Environment`.
- Trzymanie tej logiki z dala od `interpreter.rs` sprawia, że rdzeń interpretera pozostaje lekki i ogólny.

### `modules/`
- Obsługuje zewnętrzne rozszerzenia lub biblioteki standardowe ładowane przez słowo kluczowe `use` (np. `use io`).
- Obecnie służy jako placeholder dla przyszłych operacji I/O i innych modułów.

### `main.rs`
- Punkt wejścia programu. Odpowiada za CLI, czyta wejście z konsoli lub pliku i wywołuje moduły w odpowiedniej kolejności: Lexer -> Parser -> Interpreter.

### `tests.rs`
- Moduł testów jednostkowych. Używa atrybutu `#[cfg(test)]`.
- Przeprowadza testy integracyjne całego potoku: Lexer -> Parser -> Interpreter, weryfikując wyniki, struktury AST i poprawne wyrzucanie błędów (w tym operatorów logicznych, pętli, słowników, obsługi błędów i sprawdzania typów). (Obecnie 54 przechodzące testy).

## 3. Zależności
Projekt w pełni opiera się na bibliotece standardowej Rusta (`std`). Nie używa żadnych zewnętrznych skrzynek (Crate'ów).
- `std::collections::HashMap` (dla środowisk, tablic i słowników).
- `std::io` i `std::fs` (do interakcji z CLI i plikami).
- `std::env` (do czytania argumentów startowych).
