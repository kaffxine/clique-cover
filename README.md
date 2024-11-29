# Tymczasowe README: zarys projektu
## Postęp w mojej pracy
Z uwagi na autyzm i perfekcjonizm programisty back-end'u, kod jest pisany od nowa po raz czwarty. Prosimy o cierpliwość :3
## Zarys projektu
Projekt się dzieli na trzy główne części:
- front-end: /public directory
- back-end: /server directory
- microservices: /algorithms directory
Skrótem, działanie aplikacji będzie wyglądało następująco:
1. Użytkownik ustawia parametry symulacji na stronie internetowej, po czym klika przycisk "Run", wysyłając je na serwer.
2. Na serwer docierają parametry symulacji, po czym są one sprawdzane i jeśli są one poprawne, wysyłany jest request do kontenera zajmującego się generacją grafów, z odpowiednimi parametrami.
3. Po otrzymaniu zbioru grafów, serwer wysyła broadcast do sieci kontenerów z różnymi algorytmami, z których na każdym działa "wrapper", który ustanawia połączenie websocket i uruchamia instancje algorytmu dla każdego z grafów (podając zakodowany graf na stdin), jednocześnie szacując ich zużycie zasobów używając programu perf.
4. Każda z instancji algorytmu wypluwa rozwiązanie algorytmu na stdout, co jest przechwytywane przez wrappera, pakowane wraz ze statystykami dotyczącymi zużycia zasobów i wysyłane na serwer. Kontener będzie wysyłać wynik osobno dla każdej instancji, co umożliwi monitorowanie pracy programów w czasie rzeczywistym.
5. Po otrzymaniu danego pakietu danych wyjściowych algorytmu, serwer przetwarza te dane w pewien sposób (nie wiem w jaki, szczegóły w następnym paragrafie), po czym wysyła je na front-end, również za pomocą websocket.
6. Gdy każdy kontener zakończy swoją pracę, na stronie ukaże się podsumowanie z bezpośrendim porównaniem algorytmów w różnych aspektach ich działania.
## Czego jeszcze nie wiem
Rozważając rozwiązania (zbiór klik) przygotowane przez różne algorytmy dla tego samego grafu, przydałoby się jakoś ocenić ich jakość. Pierwszą myślą jest po prostu policzenie ile tych klik jest - im mniej tym lepiej. Jednak szczerze mówiąc, nie wgłębiałem się w szczegóły tego problemu (bardziej skupiając się na webdevowaniu), więc chętnie usłyszę propozycje sposobu oceny jakości rozwiązania.
## Jak przygotować algorytm
Z uwagi na modularyzację problemu i użycie technologii Docker, każdy może pisać swój algorytm w dowolnym języku programowania. Jednak ważne jest, aby algorytm wczytywał dane w określonym formacie na stdin i zapisywał dane w określonym formacie na stdout, tylko wtedy mój wrapper będzie mógł uruchomić dany algorytm i skorzystać z przygotowanych przez niego rozwiązań.
### Format danych wejściowych
Każdy algorytm na wejściu przyjmuje jeden graf, w następującym formacje:
- dwa pierwsze bajty oznaczają liczbę wierzchołków, od 0 do 2^16 - 1
- następnie, mamy macierz sąsiedztwa w wersji "trójkątnej", czyli dla n wierzchołków mamy kolejno:
- (n-1) bitów oznaczających jedynką sąsiadów pierwszego wierzchołka
- (n-2) bitów oznaczających jedynką sąsiadów drugiego wierzchołka
- (n-3) bitów [...] sąsiadów trzeciego wierzchołka
- 3 bity [...] sąsiadów (n-3)-go wierzchołka
- 2 bity [...] sąsiadów (n-2)-go wierzcholka
- 1 bit oznaczający jedynką ostatniego sąsiada (n-1)-go wierzchołka, który nie był wspomniany wcześniej
- dla n-tego wierzchołka, wspomniane zostały już wszystkie potencjalne sąsiedztwa
### Format danych wyjściowych
Tutaj sprawa wygląda o wiele prościej, gdyż mamy tu do czynienia z formatem JSON, korzystając jedynie z list. Na wyjściu programu powinna się pojawić lista list wszystkich klik, na przykład [[0,1,2,3,4],[5,6,7]] dla grafu z jedną kliką pięcio-wierzchołkową i jedną kliką trój-wierzchołkową.
