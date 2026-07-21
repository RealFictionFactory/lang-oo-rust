### Mapa drogowa rozwoju języka "Ó":

2. **Operacje na plikach (Wbudowany obiekt `File`)**
   * **Cel:** Wprowadzenie wbudowanego obiektu `File` do obsługi I/O (odczyt, zapis, dopisywanie) bez konieczności wdrażania pełnego systemu OOP (klas) w języku "Ó".
   * **Jak zrobimy:** Dodamy nowy wariant `Value::File` w interpreterze. Wprowadzimy globalną funkcję-konstruktor `file(path)`, która zwróci obiekt pliku. W obsłudze `Expr::MethodCall` dodamy specjalne ścieżki dla typu `File`, aby obsługiwać metody takie jak `read()`, `write(text)` czy `append(text)`.
   * *Pomysły/Uwagi:* Zamiast zanieczyszczać Stringi metodami plikowymi (np. `"path".read()`), używamy jawnego konstruktora. Do przemyślenia podczas implementacji: czy przechowywać w `Value::File` tylko ścieżkę (`PathBuf`) i otwierać plik przy każdej operacji (co ułatwia zarządzanie pamięcią w Ruście), czy zarządzać otwartym uchwytem (open/close).

3. **Wsparcie dla skryptowania systemowego (Shell Scripting Support)**
   * **Cel:** Umożliwienie używania języka "Ó" jako języka skryptowego w powłokach systemowych (np. Bash), w tym obsługa shebanga (`#!`), kodów wyjścia (exit codes), argumentów wiersza poleceń (CLI) oraz wywoływania komend systemowych.
   * **Jak zrobimy:** 
     * *Shebang:* W `lexer.rs` dodamy regułę ignorującą pierwszą linię pliku, jeśli zaczyna się od `#!` (do końca linii).
     * *Kody wyjścia:* W `main.rs` zmapujemy wynik działania interpretera na kody procesu (`std::process::exit(0)` dla sukcesu, `1` dla błędu). Dodamy też globalną funkcję `exit(code)` w `stdlib.rs`.
     * *Argumenty CLI:* W `stdlib.rs` dodamy globalną funkcję `args()`, która zwróci tablicę `Array` zawierającą argumenty przekazane do skryptu.
     * *Komendy systemowe:* W `stdlib.rs` dodamy funkcję `shell(command)`, która użyje `std::process::Command` do wykonania komendy w systemowym shellu i zwróci jej wynik jako `String`.

4. **Wczytywanie zewnętrznych plików `.oo` (Moduły)**
   * **Cel:** `use "math.oo"` wczyta kod z innego pliku i udostępni jego funkcje.
   * **Jak zrobimy:** Rozszerzymy `Stmt::Use`. W interpreterze, zamiast szukać modułu w wbudowanym kodzie Rusta, otworzymy plik, zlekserujemy go, sparsujemy i wykonamy w obecnym (lub globalnym) środowisku.
