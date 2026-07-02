import { createBrowserRouter } from 'react-router'

import { HomePage } from '../routes/HomePage'
import { WorkspacePage } from '../routes/WorkspacePage'

export const router = createBrowserRouter([
  {
    path: '/',
    Component: HomePage,
  },
  {
    path: '/workspace',
    Component: WorkspacePage,
  },
])
