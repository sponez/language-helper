function isApplePlatform() {
  if (typeof navigator === 'undefined') return false

  return /Mac|iPhone|iPad|iPod/i.test(
    `${navigator.platform ?? ''} ${navigator.userAgent ?? ''}`,
  )
}

export function primaryModifierLabel() {
  return isApplePlatform() ? '⌘' : 'Ctrl'
}

export function primaryAriaShortcut(key: string, shift = false) {
  const modifier = isApplePlatform() ? 'Meta' : 'Control'
  return `${modifier}+${shift ? 'Shift+' : ''}${key}`
}

export function matchesPrimaryShortcut(
  event: KeyboardEvent,
  code: string,
  shift = false,
) {
  const primaryPressed = isApplePlatform()
    ? event.metaKey && !event.ctrlKey
    : event.ctrlKey && !event.metaKey

  return (
    primaryPressed &&
    !event.altKey &&
    event.shiftKey === shift &&
    event.code === code
  )
}
