# Język "Ó" - Dokumentacja Składni

"Ó" to prosty, dynamicznie typowany język programowania z naturą zbliżoną do skryptowych. Został zaprojektowany z myślą o czytelności, braku "boilerplate'u" (np. średników) i naturalnym brzmieniu.

## 1. Podstawy
*   **Brak średników:** Koniec linii oznacza koniec instrukcji. Puste linie są ignorowane.
*   **Komentarze:** Rozpoczynają się od `//` i trwają do końca linii.
*   **Shebang:** Uniksowe shebangi (`#!/usr/bin/env ooi`) są dozwolone w pierwszej linii i ignorowane przez lekser.
*   **Bloki kodu:** Ograniczone klamrami `{ ... }`. Klamra otwierająca może być w nowej linii.

## 2. Typy Danych
Język posiada wbudowane typy, które można podawać opcjonalnie przy deklaracji:
*   `Number` - liczba całkowita (64-bitowa).
*   `Decimal` - liczba zmiennoprzecinkowa (64-bitowa).
*   `String` - ciąg znaków w podwójnych cudzysłowach.
*   `Bool` - wartość logiczna `true` lub `false`.
*   `Array` - tablica elementów. Przekazywana przez referencję (mutowalna wewnątrz funkcji).
*   `Dict` - kolekcja par klucz-wartość, gdzie kluczami są Stringi. Tworzona przy użyciu klamr `{"klucz": wartość}`. Przekazywana przez referencję.
*   `Null` - brak wartości (zwracany np. przez funkcje bez instrukcji `return` lub przy brakującym kluczu w słowniku).

## 3. Zmienne i Stałe
Deklaracja używa słów kluczowych `var` (zmienne) i `let` (stałe). Można podać typ używając `is Type`, co nada domyślną wartość (`0` dla liczb, `false` dla Bool, `""` dla String, `[]` dla Array, `{}` dla Dict).

```text
var x = 10
let pi = 3.14
var name is String  // domyślnie ""
var arr is Array // domyślnie []
```

### Sprawdzanie typów w czasie wykonania (Runtime Type Checking)
Jeśli typ zostanie jawnie określony, język egzekwuje go w czasie wykonania. Próba przypisania wartości niewłaściwego typu do otypowanej zmiennej skończy się błędem w czasie wykonania.

```text
var age is Number = 20
age = "dwadzieścia" // Błąd wykonania: Type mismatch: cannot assign String to variable of type Number
```

## 4. Operatory
*   **Matematyczne:** `+`, `-`, `*`, `/`, `%` (modulo).
*   **Jednoargumentowe:** `-` (negacja liczby, np. `-5`), `not` (negacja logiczna, np. `not true`).
*   **Logiczne:** `and`, `or` (obsługują tzw. *short-circuit evaluation*, czyli nie ewaluują prawej strony, jeśli wynik jest już znany).
*   **Porównania:** `==`, `!=`, `>`, `<`, `>=`, `<=`.
*   **Przypisania:** `=`, `+=`, `-=`. (zwraca lewą stronę, jeśli nie jest `Null`, w przeciwnym razie ewaluuje i zwraca prawą).

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

## 6. Wzorce Dopasowania (`match`)
`match` to potężna alternatywa dla długich łańcuchów `if/else if`. Może być używany jako instrukcja lub jako wyrażenie zwracające wartość. Nie występuje tu zjawisko fall-through (nie trzeba pisać `break`).

```text
var x = 2

// Jako wyrażenie
var name = match x {
    0 -> "zero"
    1 -> "jeden"
    _ -> "wiele" // _ to wildcard, łapie wszystko inne
}

// Z blokami kodu
match x {
    0 -> print("zero")
    _ -> {
        var y = x * 10
        print("wiele: ", y)
    }
}
```

## 7. Pętle (`loop`)
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

## 8. Funkcje
Definiowane słowem kluczowym `fun`. Mogą zwracać wartość używając `return`. Obsługują rekurencję i mają własny zakres zmiennych. Funkcje są "obywatelami pierwszej klasy" (First-Class Citizens) – można je przypisywać do zmiennych (lambdy), przekazywać jako argumenty i zwracać z innych funkcji (domknięcia/closures).

```text
fun add(a, b) {
  return a + b
}

// Lambda przypisana do zmiennej
var double = fun(x) { return x * 2 }

// Domknięcie (closure) pamiętające stan
fun make_counter() {
    var count = 0
    return fun() {
        count = count + 1
        return count
    }
}
var c = make_counter()
print(c()) // 1
print(c()) // 2
```

## 9. Tablice
Tworzone nawiasami kwadratowymi `[]`. Indeksowane od `0`. Przekazywane przez referencję.

```text
var arr = [1, 2, 3]
arr[0] = 99
print(arr[0]) // 99
```

## 10. Słowniki (Mapy)
Tworzone przy użyciu klamr `{}` z kluczami typu String. Dostęp do wartości i ich modyfikacja odbywa się za pomocą nawiasów kwadratowych `[]`. Przekazywane przez referencję.

Odwołanie do nieistniejącego klucza zwraca `Null` zamiast wyrzucać błąd. Można użyć operatora `??`, aby podać wartość domyślną.

```text
var user = {"name": "Jan", "age": 30}
print(user["name"]) // Jan

var role = user["role"] ?? "gość" // Zwróci "gość", bo brakuje klucza "role"
print(role)

user["age"] = 31
print(user) // {"name": "Jan", "age": 31}
```

## 11. String Interpolation
Stringi mogą zawierać wyrażenia wewnątrz `{...}`. Zostaną one obliczone i wklejone w tekst.

```text
var name = "Świecie"
var x = 5
print("Witaj {name}! Wynik to {x + 5}") // Witaj Świecie! Wynik to 10
```

## 12. Obsługa Błędów (`execute` / `onError`)
Błędy wykonania (np. dzielenie przez zero, odwołanie do nieistniejącej zmiennej lub niezgodność typów) mogą być przechwytywane za pomocą wyrażenia `execute` / `onError`. Próbuje ono wykonać pierwszy blok. Jeśli wystąpi błąd, jest on przechwytywany, komunikat błędu przypisywany do zmiennej, a następnie wykonywany jest drugi blok.

```text
var result = execute {
    10 / 0
} onError(err) {
    print("Złapano błąd: ", err)
    -1 // Zwraca wartość domyślną
}
print(result) // -1
```

## 13. Wbudowane Funkcje i Rozszerzenia

### Funkcje Globalne
*   `print(...args)` - Wypisuje argumenty na ekran oddzielone spacją.
*   `input(prompt)` - Wyświetla znak zachęty i czeka na input od użytkownika. Zawsze zwraca `String`.
*   `args()` - Zwraca `Array` typu `String` zawierający argumenty wiersza poleceń przekazane do skryptu.
*   `exit(code)` - Natychmiast kończy program z podanym kodem wyjścia (`Number`).
*   `shell(command)` - Wykonuje komendę w systemowym shellu (`cmd` na Windowsie, `sh` na Uniksie) i zwraca połączony strumień stdout/stderr jako `String`.

### Metody Rozszerzające (Stringi i Tablice)
Metody rozszerzające mogą być łańcuchowane (wywoływane jedna po drugiej).

**Metody String:**
*   `.upper()` - Zwraca string wielkimi literami.
*   `.lower()` - Zwraca string małymi literami.
*   `.trim()` - Zwraca nowy string z usuniętymi białymi znakami z początku i końca.
*   `.contains(substring)` - Zwraca `true`, jeśli string zawiera podany podciąg.
*   `.replace(old, new)` - Zwraca nowy string, w którym wszystkie wystąpienia `old` zamieniono na `new`.
*   `.split(separator)` - Zwraca `Array` stringów, powstały z podzielenia oryginalnego stringa według separatora.

**Wspólne metody Array i String:**
*   `.length()` - Zwraca długość Stringa (liczba znaków) lub tablicy (liczba elementów).
*   `.contains(element)` - Zwraca `true`, jeśli Tablica/String zawiera dany element/podciąg.

**Metody Array:**
*   `.push(element)` - Dodaje element na koniec tablicy (mutuje tablicę w miejscu).
*   `.join(separator)` - Łączy wszystkie elementy tablicy w jeden String, oddzielając je podanym separatorem.
*   `.map(fun)` - Zwraca nową tablicę, stosując podaną funkcję (lambdę) do każdego elementu.
*   `.filter(fun)` - Zwraca nową tablicę, zawierającą tylko te elementy, dla których funkcja zwróciła `true`.

### Przykład użycia input i rozszerzeń:
```text
var name = input("Jak masz na imię? ")
print("Cześć, ", name, "!")

var age = input("Ile masz lat? ").asNumber()
print("Za rok będziesz miał ", age + 1, " lat.")

var nums = [1, 2, 3, 4, 5]
var evens = nums.filter(fun(x) { return x % 2 == 0 })
print("Liczby parzyste: ", evens.join(", "))
```

## 14. Operacje na plikach (`use io`)
Operacje na plikach są dostępne po załadowaniu modułu `io`. Udostępnia on konstruktor `file()`, który zwraca obiekt `File` (implementowany pod maską jako słownik).

```text
use io

var f = file("output.txt")
if not f.exists() {
    f.write("Nowy plik\n")
}
f.append("Dopisywanie nowej linii\n")

var content = f.read()
print("Zawartość pliku:\n", content)
```
