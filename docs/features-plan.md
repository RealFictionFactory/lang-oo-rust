### Mapa drogowa rozwoju języka "Ó":

1. **Funkcje pierwszej klasy, Domknięcia i Lambdy (Closures)**
   * **Cel:** Przypisywanie funkcji do zmiennych `var f = fun(x) { return x * 2 }` i przekazywanie ich dalej.
   * **Jak zrobimy:** W parserze pozwolimy na użycie `fun` wewnątrz wyrażeń. W interpreterze naprawimy `Expr::Call`, aby potrafił wywoływać zmienne przechowujące `Value::Function`. Będzie to wymagało zmiany w środowisku, aby domknięcia "pamiętały" scope, w którym zostały stworzone.

2. **Wczytywanie zewnętrznych plików `.oo` (Moduły)**
   * **Cel:** `use "math.oo"` wczyta kod z innego pliku i udostępni jego funkcje.
   * **Jak zrobimy:** Rozszerzymy `Stmt::Use`. W interpreterze, zamiast szukać modułu w wbudowanym kodzie Rusta, otworzymy plik, zlekserujemy go, sparsujemy i wykonamy w obecnym (lub globalnym) środowisku.

3. **Rozbudowa Standardowej Biblioteki (String & Array Methods)**
   * **Cel:** Poszerzenie API o często używane operacje.
   * **Pomysły:** 
     * Dla Stringów: `.split(separator)`, `.contains(substring)`, `.trim()`, `.replace(old, new)`.
     * Dla Tablic: `.pop()`, `.contains(element)`, `.join(separator)`.
     * *Uwaga:* Metody takie jak `.map(fun)` i `.filter(fun)` zależą od wdrożenia domknięć (Punkt 1).

4. **Operator Potiku (Pipeline Operator `|>`)**
   * **Cel:** Ułatwienie czytania złożonych wywołań funkcji. Zamiast `print(filter(map(arr, fun), fun))` pisać `arr |> map(fun) |> filter(fun) |> print()`.
   * **Jak zrobimy:** Dodamy token `|>` w lexerze i nowy wariant AST `Expr::Pipeline`. W interpreterze po prostu przekażemy wynik lewej strony jako pierwszy argument prawej.

5. **Wzorce Dopasowania (Pattern Matching / `match`)**
   * **Cel:** Potężniejsza alternatywa dla długich `if/else if`, świetnie działająca ze słownikami i tablicami.
   * **Przykład:**
     ```Ó
     match x {
         0 => print("zero")
         1 => print("one")
         _ => print("many") // _ to wildcard
     }
     ```

6. **Bezpieczny dostęp do słowników (Optional Chaining / Nullish Coalescing)**
   * **Cel:** Unikanie błędów wykonania przy braku klucza w słowniku.
   * **Przykład:** `var name = user["name"] ?? "Anonymous"` (jeśli klucz nie istnieje, zwraca "Anonymous"). Albo `user?.["name"]`.

7. **Optymalizacja Wydajności (Zarządzanie Pamięcią)**
   * **Cel:** Na razie interpreter używa dużo `.clone()` na strukturach `Environment` i `Value`, co przy rekurencji lub długich pętlach zużywa dużo pamięci i czasu.
   * **Jak zrobimy:** W dalekiej przyszłości możemy zamienić `Value` i `Environment` na inteligentne wskaźniki `Rc<RefCell<T>>` (Referencje zliczane), co drastycznie zmniejszy kopiowanie. To duży refaktor, ale ważny dla wydajności.

8. **Operacje na plikach (Wbudowany obiekt `File`)**
   * **Cel:** Wprowadzenie wbudowanego obiektu `File` do obsługi I/O (odczyt, zapis, dopisywanie) bez konieczności wdrażania pełnego systemu OOP (klas) w języku "Ó".
   * **Jak zrobimy:** Dodamy nowy wariant `Value::File` w interpreterze. Wprowadzimy globalną funkcję-konstruktor `file(path)`, która zwróci obiekt pliku. W obsłudze `Expr::MethodCall` dodamy specjalne ścieżki dla typu `File`, aby obsługiwać metody takie jak `read()`, `write(text)` czy `append(text)`.
   * *Pomysły/Uwagi:* Zamiast zanieczyszczać Stringi metodami plikowymi (np. `"path".read()`), używamy jawnego konstruktora. Do przemyślenia podczas implementacji: czy przechowywać w `Value::File` tylko ścieżkę (`PathBuf`) i otwierać plik przy każdej operacji (co ułatwia zarządzanie pamięcią w Ruście), czy zarządzać otwartym uchwytem (open/close).
