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
*   `Array` - tablica elementów. Kontener, którego mutowalność zależy od sposobu deklaracji (patrz *Mutowalność i kopiowanie* poniżej).
*   `Dict` - kolekcja par klucz-wartość, gdzie kluczami są Stringi. Tworzona przy użyciu klamr `{"klucz": wartość}`. Kontener, tak jak `Array`.
*   `Null` - brak wartości (zwracany np. przez funkcje bez instrukcji `return` lub przy brakującym kluczu w słowniku).

`Number`, `Decimal`, `String` i `Bool` to proste typy wartościowe: przypisanie takiej wartości do innej zmiennej zawsze ją kopiuje. `Array` i `Dict` to kontenery i podlegają modelowi opisanemu w sekcji *Mutowalność i kopiowanie*.

## 3. Zmienne i Stałe
Deklaracja używa słów kluczowych `var` (zmienne) i `let` (stałe). Można podać typ używając `is Type`, co nada domyślną wartość (`0` dla liczb, `false` dla Bool, `""` dla String, `[]` dla Array, `{}` dla Dict).

```text
var x = 10
let pi = 3.14
var name is String  // domyślnie ""
var arr is Array // domyślnie []
```

### Sprawdzanie typów w czasie wykonania (Runtime Type Checking)
Jeśli typ zostanie jawnie określony, język egzekwuje go w czasie wykonania. Próba przypisania wartości niewłaściwego typu do otypowanej zmiennej skończy się błędem w czasie wykonania. Akceptowane są wyłącznie wbudowane nazwy typów; nieznany typ jest odrzucany już przy deklaracji, niezależnie od tego, czy podano wartość początkową.

```text
var age is Number = 20
age = "dwadzieścia" // Błąd wykonania: Type mismatch: cannot assign String to variable of type Number

var x is MadeUp = 1 // Błąd wykonania: Unknown type: MadeUp
```

### Mutowalność i kopiowanie (semantyka wartości)
Mutowalność jest cechą samego kontenera, wybieraną słowem kluczowym użytym do jego deklaracji:

*   `var` tworzy kontener **mutowalny** — można do niego dodawać elementy (`push`) i przypisywać do jego elementów.
*   `let` tworzy kontener **niemutowalny** — każda próba zmiany jego (lub czegokolwiek zagnieżdżonego w środku) kończy się błędem wykonania.

Przypisanie daje każdej nazwie **własny** kontener. Przypisanie jednego kontenera do innej zmiennej tworzy niezależną kopię, więc dwie zmienne nigdy przypadkiem nie współdzielą tego samego obiektu. Mutowalność kopii określa słowo kluczowe po lewej stronie:

```text
let xs = [1]
var ys = xs      // ys to niezależna, MUTOWALNA kopia xs
ys.push(99)
print(xs)        // [1]      - oryginał nietknięty
print(ys)        // [1, 99]

var a = [1]
var b = a        // ponownie niezależna kopia
b.push(9)
print(a)         // [1]
```

Aby uzyskać mutowalną kopię niemutowalnego kontenera, po prostu przypisz go do `var`; aby zamrozić migawkę mutowalnego, przypisz go do `let`. Nie ma osobnej metody kopiującej — wyborem jest słowo kluczowe.

Zmiana kontenera `let` jest zawsze błędem, niezależnie od tego, jak się do niego dostajemy — bezpośrednio, przez inną zmienną czy zagnieżdżony w innym kontenerze (niemutowalność jest **głęboka**):

```text
let xs = [1]
xs.push(2)              // Błąd wykonania: nie można dodać do niemutowalnej tablicy

let grid = [[1], [2]]
grid[0].push(9)        // Błąd wykonania: zagnieżdżona tablica też jest niemutowalna
```

**Parametry funkcji są jedynym wyjątkiem: są współdzielone przez referencję, a nie kopiowane.** To zamierzony sposób taniego przekazania dużego kontenera do funkcji. Niemutowalność podróżuje wraz z obiektem: kontener `let` jest tylko do odczytu wewnątrz funkcji, natomiast kontener `var` można modyfikować w miejscu, a wywołujący widzi zmianę.

```text
fun fill(target) { target.push(7) }

var xs = [1]
fill(xs)
print(xs)        // [1, 7]  - mutowalny kontener zmodyfikowany w miejscu

let ys = [1]
fill(ys)         // Błąd wykonania: niemutowalnego kontenera nie można zmienić wewnątrz funkcji
```

Kontener zwrócony z funkcji jest wiązany na nowo w miejscu wywołania, więc jego mutowalność określa słowo kluczowe wywołującego: `var out = f()` daje wynik mutowalny, `let out = f()` niemutowalny, niezależnie od tego, jak `f` go zbudowała.

## 4. Operatory
*   **Matematyczne:** `+`, `-`, `*`, `/`, `%` (modulo).
*   **Jednoargumentowe:** `-` (negacja liczby, np. `-5`), `not` (negacja logiczna, np. `not true`).
*   **Logiczne:** `and`, `or` (obsługują tzw. *short-circuit evaluation*, czyli nie ewaluują prawej strony, jeśli wynik jest już znany).
*   **Porównania:** `==`, `!=`, `>`, `<`, `>=`, `<=`.
*   **Przypisania:** `=`, `+=`, `-=`.
*   **Operator Nullish Coalescing:** `??` (zwraca lewą stronę, jeśli nie jest `Null`, w przeciwnym razie ewaluuje i zwraca prawą).

*Priorytety* (od najsłabszego do najsilniejszego): `??` → `or` → `and` → równość (`==`, `!=`) → porównania (`<`, `>`, `<=`, `>=`) → `+` `-` → `*` `/` `%` → operatory jednoargumentowe (`-`, `not`) → wartości i nawiasy. Zatem `1 < 2 + 3` znaczy `1 < (2 + 3)`, a `true or false and false` znaczy `true or (false and false)`.

*Konkatenacja:* Operator `+` łączy stringi. Jeśli połączysz String z Number/Decimal, liczba zostanie automatycznie zamieniona na tekst.

*Przepełnienie liczb całkowitych:* Arytmetyka liczb całkowitych (`Number`) wykraczająca poza zakres 64-bitowy zgłasza błąd wykonania, zamiast po cichu „zawijać się" (wrap-around).

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
Tworzone nawiasami kwadratowymi `[]`. Indeksowane od `0`. Tablica `var` jest mutowalna, tablica `let` — niemutowalna (patrz *Mutowalność i kopiowanie*). Zagnieżdżone przypisanie indeksowane jest obsługiwane.

```text
var arr = [1, 2, 3]
arr[0] = 99
print(arr[0]) // 99

var grid = [[1, 2], [3, 4]]
grid[0][1] = 99   // zagnieżdżone przypisanie
print(grid)       // [[1, 99], [3, 4]]
```

## 10. Słowniki (Mapy)
Tworzone przy użyciu klamr `{}` z kluczami typu String. Dostęp do wartości i ich modyfikacja odbywa się za pomocą nawiasów kwadratowych `[]`. Słownik `var` jest mutowalny, słownik `let` — niemutowalny (patrz *Mutowalność i kopiowanie*).

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
*   `.push(element)` - Dodaje element na koniec tablicy (mutuje tablicę w miejscu). Dozwolone tylko dla tablicy mutowalnej (`var`); wywołanie na tablicy niemutowalnej (`let`) to błąd wykonania. Metody rozszerzające sprawdzają też liczbę argumentów i zgłaszają błąd zamiast się wywalać, gdy podano ich za mało.
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
