const http = require('http')
const https = require('https')
const fs = require('fs')

const httpReq = (options, data = '', secure = true) => {
  if (typeof options === 'string') {
    secure = options.match(/^https/)
    const url = new URL(options)
    options = {
      method: 'GET',
      host: url.hostname,
      path: url.pathname + url.search,
      timeout: 10000,
      port: url.port
    }
  }
  // console.log(options)
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

/**
 *
 * @param {string} html
 */
const processHtml = html => {
  let rows = html.match(/<tr>[\s\S]*?<\/tr>/g)
  rows = rows
    .slice(1, rows.length - 1)
    .map(
      val =>
        val.match(
          /<td>(?<randomPart>[\s\S]*?)<\/td>[\s\S]*?[\s\S]*?<td>(?<name>[\s\S]*?)<\/td>[\s\S]*?[\s\S]*?<td>(?<hash>[\s\S]*?)<\/td>[\s\S]*?[\s\S]*?<td>(?<time>[\s\S]*?)<\/td>[\s\S]*?[\s\S]*?/
        )?.groups
    )
  if (rows.length <= 0) {
    throw new Error('Unexpected poll result (No blocks)')
  }
  if (rows.some(val => !val.randomPart || !val.name || !val.hash || !val.time)) {
    throw new Error('Unexpected poll result (Malformed blocks)')
  }
  return rows
}
// processHtml(fs.readFileSync('index.html').toString('utf-8'))

const urlBase = 'http://localhost:8084'

let lastBlock = null

const pollChain = async () => {
  let { data, status } = await httpReq(`${urlBase}`).promise
  if (status !== 200) throw new Error(`Unexpected status ${status}`)
  let rows = processHtml(data)
  let newLast = rows[rows.length - 1]
  if (lastBlock?.hash !== newLast.hash) {
    lastBlock = newLast
    process.stdout.write(`${lastBlock.randomPart} ${lastBlock.name} ${lastBlock.hash}\n`)
  }
}

const addBlock = async (randomPart, name) => {
  let { data, status } = await httpReq(`${urlBase}/validate?zufall=${randomPart}&name=${name}`).promise
  if (status !== 200) throw new Error(`Unexpected status ${status}`)
  // console.log(data)
  return !data.match(/ungültig/i)
}

var stdinStream = fs.createReadStream('/dev/stdin')

const noFailPoll = async () => {
  try {
    await pollChain()
  } catch (err) {}
}

const postMessage = async msg => {
  let body = JSON.stringify({ content: msg, nonce: Math.floor(Math.random() * 10 ** 15).toString(), tts: false })
  let { data, status } = await httpReq(
    {
      method: 'POST',
      host: 'discord.com',
      path: '/api/v9/channels/963934443263889418/messages',
      headers: {
        'content-type': 'application/json',
        authorization: 'mfa.fOBhYsdA221o6TObCMu-NXzm0r2cYffasO91HDT8YWJ2qzPVvYczO6jrLp0QFRJgHGPABX63dPRC-QRGks8W'
      }
    },
    body,
    true
  ).promise
  if (status !== 200) throw new Error(`Unexpected status when sending message: ${status}`)
}

const postMessageRetry = async msg => {
  let success = false
  let failureLogged = false
  while (!success) {
    try {
      await postMessage(msg)
      console.log(`_message send succeeded at ${Date.now()}`)
      success = true
    } catch (err) {
      if (!failureLogged) {
        console.log(`_message send failed at ${Date.now()}`)
        failureLogged = true
      }
      await new Promise(resolve => setTimeout(resolve, 60000))
    }
  }
}

const main = () => {
  setInterval(noFailPoll, 20000)
  noFailPoll()

  // console.log(process.env)

  stdinStream.on('data', async d => {
    console.log('_received message', d)
    postMessageRetry(d.toString('utf-8'))
    return
    let [randomPart, name] = d
      .toString('utf-8')
      .split(' ')
      .map(v => v.trim())
    // console.log(randomPart, name)

    let success = false
    while (!success)
      try {
        let isGoodBlock = await addBlock(randomPart, name)
        if (isGoodBlock) {
          noFailPoll()
        }
        success = true
      } catch (err) {
        // console.log(err)
        await new Promise(resolve => setTimeout(resolve, 5000))
      }
  })
}

main()
