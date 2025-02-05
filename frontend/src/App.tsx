import './App.css'
import { BrowserRouter, Route, Routes } from 'react-router-dom'
import Auth from './pages/Auth'
import Dashboard from './pages/Dashboard'
import Layout from './pages/Layout'

function App() {
    return (
        <BrowserRouter>
            <Routes>
                <Route path="/auth" element={<Auth />} />
                <Route element={<Layout />}>
                    <Route path="/dashboard" element={<Dashboard />} />
                </Route>
            </Routes>
        </BrowserRouter>
    )
}

export default App
