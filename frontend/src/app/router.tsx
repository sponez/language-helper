import { createBrowserRouter } from 'react-router'

import { HomePage } from '../routes/HomePage'

export const router = createBrowserRouter([
  {
    path: '/',
    Component: HomePage,
  },
])
