import { createSignal, lazy } from 'solid-js'
import init from './pkg'

import kitchen_sink_query from '../data/kitchen_sink_query.gql?raw'
import kitchen_sink_schema from '../data/kitchen_sink_schema.gql?raw'
import example_query from '../data/example_query.gql?raw'

const Minify = lazy(async () => {
  await init()
  return import('./Minify')
})

function App() {
  const querySignal = createSignal(example_query)
  const [, setQuery] = querySignal

  return (
    <div style={{ display: 'flex', 'flex-direction': 'column', gap: '1rem' }}>
      <h1>graphql-minify-rs</h1>
      <div>
        This demo page uses a WASM build of <code>graphql-minify-rs</code>.
        Examples:{' '}
        <select
          onChange={(e) => {
            switch (e.currentTarget.value) {
              case 'example':
                setQuery(example_query)
                break
              case 'kitchen-sink-query':
                setQuery(kitchen_sink_query)
                break
              case 'kitchen-sink-schema':
                setQuery(kitchen_sink_schema)
                break
            }
          }}
        >
          <option value="example" selected>
            Example query
          </option>
          <option value="kitchen-sink-query">Kitchen sink query</option>
          <option value="kitchen-sink-schema">Kitchen sink schema</option>
        </select>
      </div>
      <Minify valueSignal={querySignal} />
    </div>
  )
}

export default App
