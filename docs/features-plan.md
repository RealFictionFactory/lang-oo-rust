### Mapa drogowa rozwoju języka "Ó":

4. **Zarządzanie zasobami (Instrukcja `with`)**
   * **Cel:** Automatyczne wywoływanie metod czyszczących (np. `close()`) na obiektach (takich jak `File` czy przyszłe strumienie/gniazda sieciowe) po opuszczeniu bloku kodu, zwalniając programistę z tego obowiązku i zapobiegając wyciekom zasobów, nawet w przypadku wystąpienia błędów.
   * **Jak zrobimy:** 
     * W AST dodamy nową instrukcję `Stmt::With(Expr, String, Vec<Stmt>)` (wyrażenie inicjalizujące obiekt, nazwa zmiennej, ciało bloku).
     * W parserze rozpoznamy słowo kluczowe `with`, sparsujemy wyrażenie, słowo kluczowe `as`, nazwę zmiennej i blok `{ ... }`.
     * W interpreterze zewaluujemy wyrażenie, przypiszemy obiekt do zmiennej w obecnym scope i wykonamy blok kodu.
     * **Kluczowy element:** Niezależnie od tego, czy blok zakończył się sukcesem, czy błędem (`InterpErr::Err`), interpreter przed propagacją błędu sprawdzi, czy na obiekcie istnieje metoda `close()` (wyszukana w `extensions`) i wywoła ją.
   * *Przykład składni:*
     ```Ó
     with file("data.txt") as f {
         f.write("Hello")
         f.append("World")
     } // Tutaj interpreter automatycznie wywoła f.close()
     ```
   * *Pomysły/Uwagi:* Składnia wzorowana na Pythonie (`with ... as ...`). Idealnie współgra z mechanizmem `execute / onError`, ponieważ gwarantuje posprzątanie zasobów, zanim błąd zostanie przechwycony przez blok `onError`. Implementacja tego punktu prawdopodobnie zbiegnie się w przyszłości z rozbudową obiektu `File` o prawdziwe otwieranie strumieni (uchwytów) wymagające jawnego zamykania.


5. **Wczytywanie zewnętrznych plików `.oo` (Moduły)**
   * **Cel:** `use "math.oo"` wczyta kod z innego pliku i udostępni jego funkcje.
   * **Jak zrobimy:** Rozszerzymy `Stmt::Use`. W interpreterze, zamiast szukać modułu w wbudowanym kodzie Rusta, otworzymy plik, zlekserujemy go, sparsujemy i wykonamy w obecnym (lub globalnym) środowisku.
