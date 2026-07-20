### Mapa drogowa rozwoju języka "Ó":

4. **Funkcje pierwszej klasy, Domknięcia i Lambdy (Closures)**
   * **Cel:** Przypisywanie funkcji do zmiennych `var f = func(x) { return x * 2 }` i przekazywanie ich dalej.
   * **Jak zrobimy:** W parserze pozwolimy na użycie `func` wewnątrz wyrażeń. W interpreterze naprawimy `Expr::Call`, aby potrafił wywoływać zmienne przechowujące `Value::Function`. Będzie to wymagało zmiany w środowisku, aby domknięcia "pamiętały" scope, w którym zostały stworzone.

7. **Wczytywanie zewnętrznych plików `.oo` (Moduły)**
   * **Cel:** `use "math.oo"` wczyta kod z innego pliku i udostępni jego funkcje.
   * **Jak zrobimy:** Rozszerzymy `Stmt::Use`. W interpreterze, zamiast szukać modułu w wbudowanym kodzie Rusta, otworzymy plik, zlekserujemy go, sparsujemy i wykonamy w obecnym (lub globalnym) środowisku.
