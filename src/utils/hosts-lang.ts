import { StreamLanguage, HighlightStyle, syntaxHighlighting } from '@codemirror/language'
import { tags } from '@lezer/highlight'
import type { Extension } from '@codemirror/state'

/**
 * StreamLanguage parser for hosts file syntax.
 * Tokenizes: comments (#), IP addresses (IPv4/IPv6), domain names.
 */
const hostsStreamParser = StreamLanguage.define({
  token(stream) {
    // Skip leading whitespace
    if (stream.eatSpace()) return null

    // Comment: # to end of line
    if (stream.peek() === '#') {
      stream.skipToEnd()
      return 'comment'
    }

    // Beginning of content on line — check if this is the first token (IP address)
    if (stream.sol() || stream.pos === firstNonSpace(stream.string)) {
      // Try to match IPv4 or IPv6
      if (stream.match(/^(\d{1,3}\.){1,3}\d{1,3}/) || stream.match(/^[0-9a-fA-F:]+::[0-9a-fA-F:.]*/) || stream.match(/^::1?\b/) || stream.match(/^[0-9a-fA-F]{1,4}(:[0-9a-fA-F]{1,4}){2,7}/)) {
        return 'keyword'
      }
    }

    // Domain name (word with dots/hyphens)
    if (stream.match(/^[a-zA-Z0-9]([a-zA-Z0-9\-]*[a-zA-Z0-9])?(\.[a-zA-Z0-9]([a-zA-Z0-9\-]*[a-zA-Z0-9])?)*/)) {
      return 'string'
    }

    // Skip unknown character
    stream.next()
    return null
  },
})

function firstNonSpace(line: string): number {
  for (let i = 0; i < line.length; i++) {
    if (line[i] !== ' ' && line[i] !== '\t') return i
  }
  return line.length
}

/** Light theme highlight colors */
const hostsHighlightLight = HighlightStyle.define([
  { tag: tags.comment, color: '#00b42a' },
  { tag: tags.keyword, color: '#0070c1' },
  { tag: tags.string, color: '#a31515' },
])

/** Dark theme highlight colors */
const hostsHighlightDark = HighlightStyle.define([
  { tag: tags.comment, color: '#00b42a' },
  { tag: tags.keyword, color: '#569cd6' },
  { tag: tags.string, color: '#ce9178' },
])

export function hostsLanguage(dark: boolean): Extension[] {
  const highlight = dark ? hostsHighlightDark : hostsHighlightLight
  return [hostsStreamParser, syntaxHighlighting(highlight)]
}
