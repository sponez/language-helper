const TARGET_SAMPLE_RATE = 16_000

function writeAscii(view: DataView, offset: number, value: string) {
  for (let index = 0; index < value.length; index += 1) {
    view.setUint8(offset + index, value.charCodeAt(index))
  }
}

export async function recordingToWav(blob: Blob): Promise<Uint8Array> {
  const context = new AudioContext()
  try {
    const decoded = await context.decodeAudioData(await blob.arrayBuffer())
    const source = decoded.getChannelData(0)
    const frameCount = Math.max(
      1,
      Math.round((source.length * TARGET_SAMPLE_RATE) / decoded.sampleRate),
    )
    const pcm = new Int16Array(frameCount)
    for (let index = 0; index < frameCount; index += 1) {
      const sourcePosition = (index * decoded.sampleRate) / TARGET_SAMPLE_RATE
      const left = Math.min(source.length - 1, Math.floor(sourcePosition))
      const right = Math.min(source.length - 1, left + 1)
      const fraction = sourcePosition - left
      const sample =
        source[left] * (1 - fraction) + source[right] * fraction
      const clamped = Math.max(-1, Math.min(1, sample))
      pcm[index] = clamped < 0 ? clamped * 0x8000 : clamped * 0x7fff
    }

    const buffer = new ArrayBuffer(44 + pcm.byteLength)
    const view = new DataView(buffer)
    writeAscii(view, 0, 'RIFF')
    view.setUint32(4, 36 + pcm.byteLength, true)
    writeAscii(view, 8, 'WAVE')
    writeAscii(view, 12, 'fmt ')
    view.setUint32(16, 16, true)
    view.setUint16(20, 1, true)
    view.setUint16(22, 1, true)
    view.setUint32(24, TARGET_SAMPLE_RATE, true)
    view.setUint32(28, TARGET_SAMPLE_RATE * 2, true)
    view.setUint16(32, 2, true)
    view.setUint16(34, 16, true)
    writeAscii(view, 36, 'data')
    view.setUint32(40, pcm.byteLength, true)
    new Int16Array(buffer, 44).set(pcm)
    return new Uint8Array(buffer)
  } finally {
    await context.close()
  }
}
