### Mapa drogowa rozwoju języka "Ó":

3. **Pętle: `loop in` oraz `while`**
   * **Cel:** Iteracja po tablicach `loop element in moja_tablica { ... }` oraz klasyczna pętla warunkowa `while warunek { ... }`.
   * **Jak zrobimy:** Dodamy słowo kluczowe `in` i `while`. W parserze rozgałęzimy logikę pętli. W AST dodamy `Stmt::LoopIn` i `Stmt::While`. W interpreterze zaimplementujemy iterowanie po `Value::Array` oraz sprawdzanie warunku dopóki jest prawdziwy.

4. **Funkcje pierwszej klasy, Domknięcia i Lambdy (Closures)**
   * **Cel:** Przypisywanie funkcji do zmiennych `var f = func(x) { return x * 2 }` i przekazywanie ich dalej.
   * **Jak zrobimy:** W parserze pozwolimy na użycie `func` wewnątrz wyrażeń. W interpreterze naprawimy `Expr::Call`, aby potrafił wywoływać zmienne przechowujące `Value::Function`. Będzie to wymagało zmiany w środowisku, aby domknięcia "pamiętały" scope, w którym zostały stworzone.

5. **Słowniki / Mapy (Hash Maps)**
   * **Cel:** Struktury klucz-wartość: `var user = {"name": "Jan", "age": 30}`.
   * **Jak zrobimy:** Najtrudniejsza zmiana w parserze, bo klamry `{}` są już używane do bloków kodu. W `parse_factor` będziemy musieli sprawdzić kontekst (czy jest dwukropek `:`). Dodamy `Value::Dict(HashMap<String, Value>)` i rozszerzymy `Expr::IndexGet`, żeby działało ze stringami jako kluczami.

6. **Egzekwowanie typów (Runtime Type Checking)**
   * **Cel:** Sprawdzanie typów w czasie wykonania: `var x is Number = "string"` wyrzuci błąd.
   * **Jak zrobimy:** W interpreterze, podczas `Stmt::VarDecl` i `Stmt::Let`, jeśli podano `type_name`, sprawdzimy czy zewaluowana wartość pasuje do zadeklarowanego typu przed wstawieniem jej do środowiska.

7. **Wczytywanie zewnętrznych plików `.oo` (Moduły)**
   * **Cel:** `use "math.oo"` wczyta kod z innego pliku i udostępni jego funkcje.
   * **Jak zrobimy:** Rozszerzymy `Stmt::Use`. W interpreterze, zamiast szukać modułu w wbudowanym kodzie Rusta, otworzymy plik, zlekserujemy go, sparsujemy i wykonamy w obecnym (lub globalnym) środowisku.
