# Übungsblatt 7

## Aufgabe 2: Yubico OTP (Praxisaufgabe)

### a)

Während es im HOTP-Modus zu Desyncs zwischen Client und Server (Autenticated und Authenticator) kommen kann, wenn der Client zu viele Passwörter generiert,
ohne das eins davon beim Server ankommt (Wegen limitierter Resync-Fenster-Größe), kann ein YubiKey-OTP einfach symmetrisch
entschlüsselt werden um direkt den verwendeten Counter-Wert zur erhalten. Dies spart dem Server außerdem "Arbeit",
weil er nicht der Fenster-Größe entsprechend viele HMAC Werte berechnen muss,
sondern nur genau ein mal den OTP-Wert entschlüsseln, und seinen Inhalt auswerten muss.

### b)

/

### c)

Weil ein YubiKey sich als generische USB-Tastatur ausgibt und deshalb nur Keycodes (und nicht direkt Zeichen) senden kann,
muss er sich darauf verlassen können, dass bestimmte Keycodes zu bestimmten Zeichen gehören. Technisch nicht ist das nicht möglich,
weil das Mapping von Keycodes zu Zeichen vom Betriebssystem, abhängig vom konfigurierten Tastatur-Layout, bestimmt wird.
Der Modhex-Zeichensatz ist deshalb dafür ausgelegt, mit möglichst vielen verschiedenen Tastatur-Layouts kompatibel zu sein.

Hexadezimal kodiert lautet das erste entschlüsselte OTP:\
`e404b2f194b30100df13f300619abeb8`

### d)

Das Javascript-Programm gibt für das erste OTP folgende Ausgabe:
```js
otp_1 = {
  inOtp: 'fddeffijlujlvlgvficcdbitgjrfjkkg',
  decryptedHex: 'e404b2f194b30100df13f300619abeb8',
  fields: {
    uid: 'e404b2f194b3',  // Private ID
    useCtr: 1,            // Use counter
    tstp: 15930335,       // Timestamp
    sessionCtr: 0,        // Session counter
    rnd: 39521            // Random
  },
  checksum: 47294        // CRC (Dezimal-Darstellung)
}
```
Dabei habe ich mir erlaubt, "Timestamp low" und "Timestamp high" so wie in der yubico-Dokumentation als ein einziges 24-Bit-Feld zu lesen.

### e)

Es folgen die Ausgaben des Javascript-Programms für die einzelnen OTPs mit einigen Notizen zu ihren Bedeutungen.

```js
otp_2 = {
  inOtp: 'ifjjhvufikelireijuhkrvthbgbibret',
  decryptedHex: 'e404b2f194b301009a4df3015d0e412d',
  fields: {
    // Das OTP ist gültig für einen Key mit dieser private identity
    uid: 'e404b2f194b3',
    // Der YubiKey befindet sich nach wie vor in seiner ersten "Usage" und
    // wurde seit der Generierung des ersten OTPs nicht "neu gestartet"
    // (von einem Gerät entfernt & neu angeschlossen)
    useCtr: 1,
    // Timestamp; Zählt 8 mal pro Sekunde hoch
    // Seit der Generierung des ersten OTPs sind circa
    // (15945114-15930335)/8 =~ 1847 Sekunden (30 Minuten) vergangen
    tstp: 15945114,
    sessionCtr: 1, // Dieses ist das zweite OTP der aktuellen Usage
    rnd: 3677 // Zufälliger 16-Bit-Integer, für zusätzliche Entropie
  },
  // Prüfsumme
  checksum: 11585
}
```
```js
otp_3 = {
  inOtp: 'jcrldjbvgkhedcifuehccjjibervlkig',
  decryptedHex: 'e404b2f194b302005e62f800a7f5cee8',
  fields: {
    uid: 'e404b2f194b3',
    // Yubikey befindet sich wegen einem Neustart in einer neuen (der zweiten) Usage
    useCtr: 2,
    // Dem Timestamp lassen sich keine Infos entnehmen, weil er nach einem
    // Reset zufällig initialisiert wird.
    tstp: 16278110,
    // Erstes OTP der Usage
    sessionCtr: 0,
    rnd: 62887
  },
  checksum: 59598
}
```
```js
otp_4 = {
  inOtp: 'hfkfkrcnjeufivlhgjecckuhenlgngjg',
  decryptedHex: 'e404b2f194b30300b409d5005fe00bd5',
  fields: {
    uid: 'e404b2f194b3',
    // Dritte Usage
    useCtr: 3,
    tstp: 13961652,
    // Erstes OTP der Usage
    sessionCtr: 0,
    rnd: 57439
  },
  checksum: 54539
}
```
```js
otp_5 = {
  inOtp: 'uhhlevhvttturkigguefuhndckukhfkh',
  decryptedHex: 'e404b2f194b30300c209d501cc372bc3',
  fields: {
    uid: 'e404b2f194b3',
    // Noch gleiche Usage
    useCtr: 3,
    // Seit dem letzten OTP sind (13961666-13961652)/8 = 1,75 Sekunden vergangen
    tstp: 13961666,
    // Zweites OTP der Usage
    sessionCtr: 1,
    rnd: 14284
  },
  checksum: 49963
}
```
```js
otp_6 = {
  inOtp: 'tugkifjrcjkthbbrerrrkeujebkbiiku',
  decryptedHex: 'e404b2f194b30300db09d502717bf249',
  fields: {
    uid: 'e404b2f194b3',
    useCtr: 3,
    // Seit dem letzten OTP sind (13961691-13961666)/8 = 3,125 Sekunden vergangen
    tstp: 13961691,
    // Drittes OTP der Usage
    sessionCtr: 2,
    rnd: 31601
  },
  checksum: 18930
}
