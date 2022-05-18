const crypto = require('crypto')

const decrypt = (bytes, key) => {
  const decipher = crypto.createDecipheriv('aes-128-ecb', key, null)
  decipher.setAutoPadding(false) // Not sure why this is necessary.
  return Buffer.concat([decipher.update(bytes), decipher.final()])
}

const modhexCharset = 'cbdefghijklnrtuv'
const hexCharset = '0123456789abcdef'
const modhexHexMap = modhexCharset
  .split('')
  .reduce((acc, val, index) => [(acc[val] = hexCharset[index]), acc][1], {})
const modhexToHex = modhex =>
  modhex
    .split('')
    .map(ch => modhexHexMap[ch])
    .join('')

const interpretOtp = (keyHex, otpModhex) => {
  const key = Buffer.from(keyHex, 'hex')
  otpHex = modhexToHex(otpModhex)
  const otp = Buffer.from(otpHex, 'hex')

  const decrypted = decrypt(otp, key)

  const copyBufferPart = (start, end) => Buffer.from(new Uint8Array(decrypted).slice(start, end))

  return {
    inOtp: otpModhex,
    decryptedHex: decrypted.toString('hex'),
    fields: {
      uid: copyBufferPart(0, 6).toString('hex'),
      useCtr: decrypted.readUint16LE(6),
      tstp: Buffer.concat([copyBufferPart(8, 8 + 3), Buffer.from([0])]).readUint32LE(0),
      sessionCtr: decrypted.readUint8(11),
      rnd: decrypted.readUint16LE(12)
    },
    checksum: decrypted.readUint16LE(14)
  }
}

const keyHex = '01436f54acf94360f18f5b1378c3f996'
const otps = [
  'fddeffijlujlvlgvficcdbitgjrfjkkg',
  'ifjjhvufikelireijuhkrvthbgbibret',
  'jcrldjbvgkhedcifuehccjjibervlkig',
  'hfkfkrcnjeufivlhgjecckuhenlgngjg',
  'uhhlevhvttturkigguefuhndckukhfkh',
  'tugkifjrcjkthbbrerrrkeujebkbiiku'
]

const main = () => {
  for (let [i, otp] of otps.entries()) {
    console.log('Interpretation of otp', i + 1)
    console.log(interpretOtp(keyHex, otp))
  }
}

main()
