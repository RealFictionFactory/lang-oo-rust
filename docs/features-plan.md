### Mapa drogowa rozwoju języka "Ó":

4. **Funkcje pierwszej klasy, Domknięcia i Lambdy (Closures)**
   * **Cel:** Przypisywanie funkcji do zmiennych `var f = func(x) { return x * 2 }` i przekazywanie ich dalej.
   * **Jak zrobimy:** W parserze pozwolimy na użycie `func` wewnątrz wyrażeń. W interpreterze naprawimy `Expr::Call`, aby potrafił wywoływać zmienne przechowujące `Value::Function`. Będzie to wymagało zmiany w środowisku, aby domknięcia "pamiętały" scope, w którym zostały stworzone.

6. **Egzekwowanie typów (Runtime Type Checking)**
   * **Cel:** Sprawdzanie typów w czasie wykonania: `var x is Number = "string"` wyrzuci błąd.
   * **Jak zrobimy:** W interpreterze, podczas `Stmt::VarDecl` i `Stmt::Let`, jeśli podano `type_name`, sprawdzimy czy zewaluowana wartość pasuje do zadeklarowanego typu przed wstawieniem jej do środowiska.

7. **Wczytywanie zewnętrznych plików `.oo` (Moduły)**
   * **Cel:** `use "math.oo"` wczyta kod z innego pliku i udostępni jego funkcje.
   * **Jak zrobimy:** Rozszerzymy `Stmt::Use`. W interpreterze, zamiast szukać modułu w wbudowanym kodzie Rusta, otworzymy plik, zlekserujemy go, sparsujemy i wykonamy w obecnym (lub globalnym) środowisku.
