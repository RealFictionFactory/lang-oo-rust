# Język "Ó" - Dokumentacja Składni

"Ó" to prosty, dynamicznie typowany język programowania z naturą zbliżoną do skryptowych. Został zaprojektowany z myślą o czytelności, braku "boilerplate'u" (np. średników) i naturalnym brzmieniu.

## 1. Podstawy
*   **Brak średników:** Koniec linii oznacza koniec instrukcji. Puste linie są ignorowane.
*   **Komentarze:** Rozpoczynają się od `//` i trwają do końca linii.
*   **Bloki kodu:** Ograniczone klamrami `{ ... }`. Klamra otwierająca może być w nowej linii.

## 2. Typy Danych
Język posiada wbudowane typy, które można podawać opcjonalnie przy deklaracji:
*   `Number` - liczba całkowita (64-bitowa).
*   `Decimal` - liczba zmiennoprzecinkowa (64-bitowa).
*   `String` - ciąg znaków w podwójnych cudzysłowach.
*   `Bool` - wartość logiczna `true` lub `false`.
*   `Array` - tablica elementów.
*   `Dict` - kolekcja par klucz-wartość, gdzie kluczami są Stringi. Tworzona przy użyciu klamr `{"klucz": wartość}`.
*   `Null` - brak wartości (zwracany np. przez funkcje bez instrukcji `return`).

## 3. Zmienne i Stałe
Deklaracja używa słów kluczowych `var` (zmienne) i `let` (stałe). Można podać typ używając `is Type`, co nada domyślną wartość (`0` dla liczb, `false` dla Bool, `""` dla String).

```text
var x = 10
let pi = 3.14
var name is String  // domyślnie ""
var counter is Number // domyślnie 0
```

## 4. Operatory
*   **Matematyczne:** `+`, `-`, `*`, `/`, `%` (modulo).
*   **Jednoargumentowe:** `-` (negacja liczby, np. `-5`), `not` (negacja logiczna, np. `not true`).
*   **Logiczne:** `and`, `or` (obsługują tzw. *short-circuit evaluation*, czyli nie ewaluują prawej strony, jeśli wynik jest już znany).
*   **Porównania:** `==`, `!=`, `>`, `<`, `>=`, `<=`.
*   **Przypisania:** `=`, `+=`, `-=`.

*Konkatenacja:* Operator `+` łączy stringi. Jeśli połączysz String z Number/Decimal, liczba zostanie automatycznie zamieniona na tekst.

## 5. Instrukcje Warunkowe (`if` / `else`)
`if` może być użyte jako standardowa instrukcja lub jako wyrażenie, które zwraca wartość.

```text
var x = 5
if x > 10 {
  print("Dużo")
} else if x == 5 {
  print("Pięć")
} else {
  print("Mało")
}

// if jako wyrażenie
var status = if x > 10 { "dużo" } else { "mało" }
```
*Prawdziwość (Truthiness):* W warunkach wartości `0`, `0.0`, `""` (pusty string) i `false` są traktowane jako fałsz. Wszystko inne to prawda.

## 6. Pętle (`loop`)
Słowo kluczowe `loop` jest bardzo wszechstronne i obsługuje kilka wariantów:

### Pętla zakresowa (Range Loop)
Iteruje po liczbach całkowitych (`Number`). Wykorzystuje słowo kluczowe `from` i zakres `..`.
```text
loop i from 1..5 {
  if i == 3 { continue } // pomija 3
  print(i)
}
```

### Pętla po tablicy (Array Iteration Loop)
Iteruje po elementach tablicy. Wykorzystuje słowo kluczowe `in`.
```text
var arr = [10, 20, 30]
loop element in arr {
  print(element)
}
```

### Pętla nieskończona / warunkowa (`loop {}` oraz `until`)
Zwykłe `loop {}` tworzy pętlę nieskończoną. Można ją przerwać używając `break`, lub warunkowo przerwać używając `until (warunek)`.
`until` działa jak warunkowy `break`. Jeśli umieścisz go na górze bloku, działa jak `while not`. Jeśli na dole, działa jak `do-while not` (wykonuje się przynajmniej raz).

```text
// Działa jak pętla do-while
var pass = ""
loop {
  pass = input("Podaj hasło: ")
  until (pass == "tajne") // Sprawdza na końcu bloku
}
```

## 7. Funkcje
Definiowane słowem kluczowym `fun`. Mogą zwracać wartość używając `return`. Obsługują rekurencję i mają własny zakres zmiennych (zmienne wewnątrz funkcji nie nadpisują zmiennych globalnych).

```text
fun add(a, b) {
  return a + b
}

fun factorial(n) {
  if n <= 1 {
    return 1
  }
  return n * factorial(n - 1)
}

print(factorial(5)) // 120
```

## 8. Tablice
Tworzone nawiasami kwadratowymi `[]`. Indeksowane od `0`.

```text
var arr = [1, 2, 3]
arr[0] = 99
print(arr[0]) // 99
```

## 9. Słowniki (Mapy)
Tworzone przy użyciu klamr `{}` z kluczami typu String. Dostęp do wartości i ich modyfikacja odbywa się za pomocą nawiasów kwadratowych `[]`.

```text
var user = {"name": "Jan", "age": 30}
print(user["name"]) // Jan
user["age"] = 31
print(user) // {"name": "Jan", "age": 31}
```

## 10. String Interpolation
Stringi mogą zawierać wyrażenia wewnątrz `{...}`. Zostaną one obliczone i wklejone w tekst.

```text
var name = "Świecie"
var x = 5
print("Witaj {name}! Wynik to {x + 5}") // Witaj Świecie! Wynik to 10
```

## 11. Obsługa Błędów (`execute` / `onError`)
Błędy wykonania (np. dzielenie przez zero lub odwołanie do nieistniejącej zmiennej) mogą być przechwytywane za pomocą wyrażenia `execute` / `onError`. Próbuje ono wykonać pierwszy blok. Jeśli wystąpi błąd, jest on przechwytywany, komunikat błędu przypisywany do zmiennej, a następnie wykonywany jest drugi blok.

```text
var result = execute {
    10 / 0
} onError(err) {
    print("Złapano błąd: ", err)
    -1 // Zwraca wartość domyślną
}
print(result) // -1
```

## 12. Wbudowane Funkcje i Rozszerzenia

### Funkcje Globalne
*   `print(...args)` - Wypisuje argumenty na ekran oddzielone spacją.
*   `input(prompt)` - Wyświetla znak zachęty i czeka na input od użytkownika. Zawsze zwraca `String`.

### Metody Rozszerzające
Metody rozszerzające mogą być łańcuchowane (wywoływane jedna po drugiej).

**Konwersja Stringów:**
*   `.asNumber()` - Konwertuje String na `Number` (liczba całkowita).
*   `.asDecimal()` - Konwertuje String na `Decimal` (liczba zmiennoprzecinkowa).
*   `.asBoolean()` - Konwertuje String na `Bool` (rozpoznaje "true"/"1" jako true, "false"/"0" jako false).

**Metody String:**
*   `.upper()` - Zwraca string wielkimi literami.
*   `.lower()` - Zwraca string małymi literami.

**Wspólne metody Array i String:**
*   `.length()` - Zwraca długość Stringa (liczba znaków) lub tablicy (liczba elementów).

**Metody Array:**
*   `.push(element)` - Dodaje element na koniec tablicy (mutuje tablicę w miejscu).

### Przykład użycia input i rozszerzeń:
```text
var name = input("Jak masz na imię? ")
print("Cześć, ", name, "!")

var age = input("Ile masz lat? ").asNumber()
print("Za rok będziesz miał ", age + 1, " lat.")

let shout = input("Powiedz coś cicho: ").upper()
print("KRZYCZĘ: ", shout)
```
