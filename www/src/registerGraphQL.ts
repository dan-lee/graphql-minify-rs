import { languages } from 'prism-code-editor'

languages.graphql = languages.gql = {
  comments: {
    block: ['"""', '"""'],
    line: '#',
  },
  autoIndent: [
    ([start], value) =>
      /[([{][^\n)\]}]*$/.test(value.slice(0, start).slice(-999)),
    ([start, end], value) => /\[]|\(\)|{}/.test(value[start - 1] + value[end]),
  ],
}
