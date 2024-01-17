import { createApp } from 'vue'
import { createPinia } from 'pinia'
import { createRouter, createWebHistory } from 'vue-router'
import App from './App.vue'
import home from './components/Home.vue'
import './index.css'

const routes = [
    { path: "/", component: home},
    { path: "/login", component: () => import('./components/Login.vue') }
]

const router = createRouter({
    history: createWebHistory(),
    routes
})

const pinia = createPinia()

const app = createApp(App)
app.use(pinia)
app.use(router)
app.mount("#app")
