# Übungsblatt 1 (2022-03-24)

## Aufgabe 1

Mit 26 zeichen (Case-insensitive):\
26⁸ = 208.827.064.576

Mit 52 Zeichen (Case-sensitive):\
52⁸ = 53.459.728.531.456

52⁸ / 28⁸ = 173.056

**Mit dem größeren Zeichensatz dauert eine Brute Force Attacke im Worst-Case 173.056 mal länger als mit dem kleineren.**

## Aufgabe 2

Überlegung: Aus einem gegebenen Merkzettel kommen für die "Lösung" nur Buchstaben in Frage, die in genau einer Spalte vorkommen. Ein Buchstabe, der in mehreren Spalten vorkommt, kann beim "entschlüsseln" nicht eindeutig einer Ziffer zugeordnet werden. Deshalb muss eine Nutzer\*in des Systems beim Erstellen der Tabelle darauf achten, _mindestens_ die im Referenz-Wort vorkommenden Buchstaben nur in einer Spalte zu verwenden.

Mit Buchstaben entfernt, die in mehreren Spalten (Hier Zeilen) auftauchen, sieht die gegebene Tabelle so aus:

```
0 - rc
1 - x
2 - h
3 - yq
4 - p
5 - t
6 - nüo
7 -
8 -
9 - z b
```

Für das erstellen der Tabelle können nur Wörter verwendet worden sein, die ausschließlich aus den übrig bleibenden Buchstaben bestehen. Folgender regulärer Ausdruck erlaubt das durchsuchen des Wörterbuchs nach diesen Wörtern:

`/^[rcxhyqptnüozb]\*$/`

Die Suche ergibt folgende 44 Wörter:

```
bohr, bonn, boot, born, boxt, brot, büro, bütt, chor, coop, hobt, hoch, hohn, hopp, horn, hort, http, noch, onyx, otto, ozon, phon, pony, popo, port, porz, pott, pütt, roch, rohr, roth, rott, rotz, rühr, thor, tobt, tony, topp, torr, tory, toto, trüb, xbox, zorn
```

... die als Ausgangswort genutzt worden sein könnten.

Dies ist aber nicht unbedingt die Anzahl der möglichen PINs, da mehrere Wörter mit der Tabelle die gleiche Ziffernfolge ergeben können. Mit search-and-replace können die Wörter anhand der Tabelle in ihre zugehörigen Pins umgewandelt werden.

Nach dem entfernen aller Duplikate bleibt eine Liste von 40 möglichen PINs. Diese lautet wie folgt:

```
9620, 9666, 9665, 9606, 9615, 9065, 9655, 0260, 0664, 2695, 2602, 2626, 2644, 2606, 2605, 2554, 6602, 6631, 6556, 6966, 4266, 4663, 4646, 4605, 4609, 4655, 0602, 0620, 0652, 0655, 0659, 5260, 5695, 5663, 5644, 5600, 5603, 5656, 5069, 1961
```

**Eine beliebige Auswahl von 3 unterschiedlichen PINs aus dieser Liste hat eine Wahrscheinlichkeit von 3/40=0,075, die korrekte zu beinhalten. Mit drei Versuchen ist die Chance, richtig zu raten, 7,5%**.

## Aufgabe 3

### a)

Siehe main.js (Node.js v16)

### b)

#### Im Campus-Netzwerk (HTTPS)

1 Request zur Zeit: ~13 Requests pro Sekunde\
10 aktive Requests zu jedem Zeitpunkt: ~100 /s\
100: ~400 /s\
1000: ~480 /s\
10000: <400 /s

=> Diminishing Returns und sogar Verschlechterung bei sehr extremer Concurrency
(Vermutlich hauptsächlich wegen langer Wartezeiten, bevor Requests überhaupt abgesendet werden, weil das Betriebssystem nicht beliebig viele TCP-Verbindungen gleichzeitig aufbauen kann)

#### Im Campus-Netzwerk (HTTP)

500 aktive Requests zu jedem Zeitpunkt: ~680 Requests pro Sekunde

=> Schneller als mit HTTPS

#### Aus dem Internet

Aus dem Internet ist der Zugriff sehr langsam, und viele Requests gleichzeitig (>100) führen zu größeren Problemen (Einige werden scheinbar nie beantwortet). Ich konnte nicht mehr als 80 Requests pro Sekunde erreichen.

#### Mögliche Bottlenecks und Problemursachen (Auf der Client-Seite):

Zwar async aber single-threaded (Javascript) (Dürfte kein großes Problem sein, da das Programm die meiste Zeit sowieso nur asynchron auf Betriebssystems-I/O wartet)\
Limitierte Range von ephemeral TCP Ports (Mit mehr IP-Adressen ließen sich eventuell mehr Verbindungen aufbauen)\
Ich weiß nicht, wie die node.js http/https Module genau implementiert sind. Könnten ineffizient sein, wenn sie z.B. TCP Verbindungen nicht so häufig wie möglich wieder-verwenden.

Eventuell stören auch die Clients anderer Studierender, die gleichzeitig mit mir den Webserver belasten und Antwort-Zeiten verlängern.

### c)

Die Passwörter für `bob`, `ute` und `joe` konnte ich ermitteln:

```
bob: nY
ute: G0j
joe: dreamweaver
```

Ich hatte nicht genug Zeit, um das 4-Zeichen-Lange Passwort zu bruteforcen. Bei meinen im Schnitt 60 Requests pro Sekunde aus dem Internet hätte der Bruteforce im Worst-Case für 14 Mio Versuche ~70 Stunden gedauert; Ich hätte entweder meinen Ansatz weiter optimieren müssen, oder einfach für 8 Stunden auf dem Campus sitzen müssen. Ein einfacher Mechanismus, um zu persistieren, welche Passwörter bereits ausprobiert wurden, wäre auch sinnvoll. Dann würde das Programm nicht bei jedem Aufruf von vorn Anfangen.

### d)

Genutzt werden können Request-Rate-Limiting und Anforderungen an Passwort-Länge und Komplexität, um einfache Brute-Force-Angriffe zu erschweren.\
Auch können Login-Bestätigungs-Emails und 2FA implementiert werden, die zwar das erraten von Passwörtern nicht erschweren, aber es weniger nützlich machen.
