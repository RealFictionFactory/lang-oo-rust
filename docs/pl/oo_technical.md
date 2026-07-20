# Język "Ó" - Dokumentacja Techniczna

Ten dokument opisuje architekturę interpretera języka "Ó" napisanego w języku Rust.

## 1. Architektura
Interpreter implementuje klasyczny proces tłumaczenia kodu na wykonanie:
`Kod źródłowy (String)` -> **Lekser** -> `Tokeny` -> **Parser** -> `AST` -> **Interpreter** -> `Wykonanie`

Program działa w dwóch trybach:
*   **REPL:** Interaktywna konsola uruchamiana poleceniem `cargo run`. Znak zachęty `ó>`. Kod wykonuje się po wpisaniu `.` w pustej linii.
*   **Plikowy:** Uruchamiany przez `cargo run -- plik.oo`. Czyta i wykonuje cały plik.

## 2. Moduły (Struktura Projektu)

Projekt jest podzielony na logiczne moduły w folderze `src/`:

### `lexer.rs`
- Odpowiada za analizę leksykalną.
- Zmienia ciąg znaków (`String`) na wektor tokenów (`Vec<Token>`).
- Rozpoznaje słowa kluczowe, literały (Number, Decimal, String, Bool), identyfikatory i operatory.
- Posiada logikę "podglądania" (peek) dla operatorów dwuznakowych (np. `==`, `+=`, `..`, `<=`).
- Automatycznie rozpoznaje ułamki, sprawdzając czy po liczbie stoi kropka, która nie jest początkiem zakresu `..`.
- Ignoruje komentarze `//` aż do końca linii, generując tokeny `NewLine` znaczące koniec instrukcji.

### `ast.rs`
- Zawiera definicje struktury drzewa Abstract Syntax Tree (AST).
- `Expr`: Wyrażenia, które po obliczeniu zwracają wartość (np. matematyka, dostęp do zmiennej, wywołanie funkcji).
- `Stmt`: Instrukcje, które robią coś w kodzie (deklaracje, pętle, przypisania), ale nie zwracają wartości.
- Drzewa rekurencyjne są opakowane w `Box<T>` z racji wymogów Rusta dot. rozmiaru struktur.

### `parser.rs`
- Implementuje technikę *Recursive Descent Parsing*.
- Buduje AST na podstawie `Vec<Token>` zwróconego przez lekser.
- Rozdziela parsowanie wyrażeń na poziomy priorytetów: `parse_expr` (+, -, porównania) -> `parse_term` (*, /, %) -> `parse_factor` (wartości, nawiasy).
- Używa mechanizmu desugaringu: np. `+=` zamienia na drzewo przypisania z dodawaniem w locie, a interpolacja stringów `"a {b}"` tworzy wewnętrzną instancję Leksera i Parsera by zwrócić wyrażenie `Expr::Binary("a " + b)`.
- Wywołanie funkcji jest parsowane "postfiksowo" na końcu `parse_factor`.

### `interpreter.rs`
- Implementuje logikę wykonania na drzewie AST.
- Główną strukturą jest `Environment` (zastąpiła płaską `HashMap`), implementująca wzorzec łańcucha zasięgów (Scope Chain) z polem `parent: Option<Box<Environment>>`. To pozwala funkcjom czytać zmienne globalne, nie nadpisując ich trwale swoimi lokalnymi.
- Wykorzystuje własny system błędów `InterpErr` z wariantami `Return`, `Break` i `Continue`. Dzięki wykorzystaniu propagacji błędów (`?`), instrukcje te "przebijają się" przez zagnieżdżone bloki aż do pętli lub ciała funkcji, co elegancko rozwiązuje problem sterowania przepływem bez flag.
- Funkcje wbudowane (`print`, `len`, `push`) przechowywane są jako wskaźniki na funkcje Rustowe `BuiltinFn` wewnątrz enuma `Value`.

### `main.rs`
- Punkt wejścia programu. Odpowiada za CLI, czytanie wejścia z konsoli lub pliku i wywoływanie w odpowiedniej kolejności modułów: Lexer -> Parser -> Interpreter.

### `tests.rs`
- Moduł testów jednostkowych. Używa atrybutu `#[cfg(test)]`.
- Przeprowadza testy integracyjne całego potoku: Lexer -> Parser -> Interpreter, weryfikując wyniki oraz poprawność wyrzucania błędów. (Obecnie 32 passing tests).

## 3. Zależności
Projekt w pełni opiera się na bibliotece standardowej Rusta (`std`). Nie używa żadnych zewnętrznych skrzynek (Crate'ów).
- `std::collections::HashMap` (do środowisk i tablic).
- `std::io` i `std::fs` (do interakcji z CLI i plikami).
- `std::env` (do czytania argumentów startowych).
