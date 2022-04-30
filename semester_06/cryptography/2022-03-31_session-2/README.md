# Übungsblatt 2

## Aufgabe 2



## Aufgabe 3

### a)

Das einfachste (und wahrscheinlich schnellste!) Computerprogramm, dass für die Passwörter von Länge 1 bis 8 mit dem Zeichensatz a-z,A-Z,0-9 diese Aufgabe löst, ist ein Programm, das wie folgt Hashcat ausführt:

```bash
hashcat -O -m 100 -a 3 --increment -o out.txt hashed.txt -1 "?l?u?d" "?1?1?1?1?1?1?1"
```

... und ein zweites mal Hashcat ausführt, um via Wörterbuch-Angriff das letzte Passwort zu finden:

```
hashcat -O -m 100 -a 0 -o out.txt hashed.txt dictionary.txt
```

Um einen Streit um die Definition des Wortes "Computerprogramm" zu vermeiden, habe ich zusätzlich ein [Rust-Programm](task_2) geschrieben, welches in mehreren Threads auf der CPU rechnet.

### b)

Hashcat erreicht auf meiner GPU ~11300 MH/s, also 11,3 Milliarden SHA-1 Hashes pro Sekunde.\
Mein Rust-Programm erreicht auf meiner CPU mit 16 threads ungefähr 160.000.000 Hashes pro Sekunde.

### c)

Die Passwörter lauten wie folgt:

| Username | SHA-1 Hash                                 | Klartext Passwort |
| :------- | :----------------------------------------- | :---------------- |
| bob      | `079f8191fe2fc4b01bb6415083db2ed481b7ec32` | `nY`              |
| ute      | `db23fe065e9f857e4cd3398a25299be0bc383c2b` | `G0j`             |
| paul     | `4a660a7d88dbde0b75dd2f6399e23226c259b7ff` | `Q2N9`            |
| nina     | `e01a18a0d1b0dbe455c56de57079f52015554f68` | `1Kal8`           |
| anja     | `9fa154f3a0baa0aadf70066f1f4dbd62258b1c99` | `ShfdZ0`          |
| fritz    | `ff5cf374186912339aaa14b73e90f1545d43aa96` | `ZuYqZfy`         |
| peter    | `fd7e698d04cad5a2a20a9256cbf929aee58732e9` | `TMCr5Nzg`        |
| joe      | `06a12c67249567c66725263ac26d5f508448f1e1` | `dreamweaver`     |

### d)

* Man kann eine Hash-Funktion verwenden, die speziell designt ist, um insbesondere auf GPUs und anderer spezialisierter Hardware teuer bzw. schwer zu implementieren zu sein. (z.B. scrypt)
* Auch können Brute-Force-Angriffe dadurch erschwert werden, eine beliebige Hash-Funktion mehrmals anzuwenden.\
* Die Nutzung eines Salts erschwert "Sammel-Angriffe" auf mehrere oder viele Hashes und macht die Nutzung von vorberechneten Hash-Tables unpraktikabel.