import {
  Signal,
  createEffect,
  onMount,
  on,
  createSignal,
  createMemo,
} from 'solid-js'
import Prism from 'prism-code-editor/prism-core'
import 'prismjs/components/prism-graphql.js'

import { PrismEditor, createEditor, languages } from 'prism-code-editor'
import { matchBrackets } from 'prism-code-editor/match-brackets'
import { indentGuides } from 'prism-code-editor/guides'
import { highlightSelectionMatches } from 'prism-code-editor/search'
import { defaultCommands } from 'prism-code-editor/commands'
import { cursorPosition } from 'prism-code-editor/cursor'
import 'prism-code-editor/layout.css'
import 'prism-code-editor/scrollbar.css'
import 'prism-code-editor/themes/github-dark.css'
import { stripIgnoredCharacters } from 'graphql'

import './registerGraphQL.ts'
import { minify } from './pkg'

export default function Minify({
  valueSignal,
}: {
  valueSignal: Signal<string>
}) {
  let inputRef: HTMLDivElement | undefined
  let outputRef: HTMLDivElement | undefined
  let inputEditor: PrismEditor | undefined
  let outputEditor: PrismEditor | undefined

  const [value, setValue] = valueSignal
  const [matchesJSImpl, setMatchesJSImpl] = createSignal(true)

  const [minified, setMinified] = createSignal('')

  createEffect(
    on(value, (value) => {
      inputEditor?.setOptions({ value })

      let minified = ''
      try {
        minified = minify(value)
        const jsMinified = stripIgnoredCharacters(value)

        setMatchesJSImpl(minified === jsMinified)
      } catch (e) {
        console.log(e)
        minified = `Could not minify document`
      }
      setMinified(minified)
    }),
  )

  createEffect(
    on(minified, (value) => {
      outputEditor?.setOptions({ value })
    }),
  )

  onMount(() => {
    const cursor = cursorPosition()
    inputEditor = createEditor(
      Prism,
      inputRef!,
      {
        language: 'graphql',
        value: value(),
        onUpdate: (value: string) => setValue(value),
      },
      indentGuides(),
      matchBrackets(),
      highlightSelectionMatches(),
      defaultCommands(cursor),
      cursor,
    )

    outputEditor = createEditor(
      Prism,
      outputRef!,
      {
        language: 'graphql',
        value: minify(value()),
        wordWrap: true,
        readOnly: true,
      },
      highlightSelectionMatches(),
      matchBrackets(),
      cursorPosition(),
    )
  })

  const saved = createMemo(
    () => ((value().length - minified().length) / value().length) * 100,
  )

  return (
    <div>
      Saved:{' '}
      {isNaN(saved())
        ? 'N/A'
        : `${saved().toFixed(1)}% (${value().length} chars
      → ${minified().length} chars)`}{' '}
      - Matches GraphQL.js:{' '}
      {matchesJSImpl() ? (
        '✅'
      ) : (
        <>
          ❌ (this should not happen! <a href="">please report a bug issue!</a>)
        </>
      )}
      <div id="editor-container">
        <div ref={inputRef} />
        <div ref={outputRef} />
      </div>
    </div>
  )
}
