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
*   `Null` - brak wartości (zwracany np. przez funkcje bez `return`).

## 3. Zmienne i Stałe
Deklaracja używa słów kluczowych `var` (zmienne) i `let` (stałe). Można podać typ używając `is Type`, co nada domyślną wartość (0 dla liczb, `false` dla Bool, `""` dla String).

```text
var x = 10
let pi = 3.14
var name is String  // domyślnie ""
var counter is Number // domyślnie 0
```

## 4. Operatory
*   **Matematyczne:** `+`, `-`, `*`, `/`, `%` (modulo).
*   **Porównania:** `==`, `!=`, `>`, `<`, `>=`, `<=`.
*   **Przypisania:** `=`, `+=`, `-=`.

*Konkatenacja:* Operator `+` łączy stringi. Jeśli połączysz String z Number/Decimal, liczba zostanie automatycznie zamieniona na tekst.

## 5. Instrukcje Warunkowe (`if` / `else`)
```text
var x = 5
if x > 10 {
  print("Dużo")
} else if x == 5 {
  print("Pięć")
} else {
  print("Mało")
}
```
*Prawdziwość (Truthiness):* W warunkach `if`, wartość `0`, `0.0`, `""` (pusty string) i `false` są traktowane jako fałsz. Wszystko inne to prawda.

## 6. Pętle (`loop`)
Pętla iteruje po liczbach całkowitych (Number). Wykorzystuje słowo kluczowe `from` i zakres `..`.

```text
loop i from 1..5 {
  if i == 3 {
    continue // pomija 3
  }
  print(i)
}

loop j from 0..10 {
  if j == 5 {
    break // przerywa pętlę
  }
}
```

## 7. Funkcje
Definiowane słowem kluczowym `func`. Mogą zwracać wartość używając `return`. Obsługują rekurencję i mają własny zakres zmiennych (zmienne wewnątrz funkcji nie psują zmiennych globalnych).

```text
func add(a, b) {
  return a + b
}

func factorial(n) {
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

## 9. String Interpolation
Stringi mogą zawierać wyrażenia wewnątrz `{...}`. Zostaną one obliczone i wklejone w tekst.

```text
var name = "Świecie"
var x = 5
print("Witaj {name}! Wynik to {x + 5}") // Witaj Świecie! Wynik to 10
```

## 10. Wbudowane Funkcje
*   `print(...args)` - Wypisuje argumenty na ekran oddzielone spacją.
