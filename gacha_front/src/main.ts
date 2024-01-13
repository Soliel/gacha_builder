import './assets/main.css'

import { createApp } from 'vue'
import { createPinia } from 'pinia'
import VueRouter from 'vue-router'
import App from './App.vue'
import home from './components/Home.vue'

const routes = [
    { path: "/", component: home},
    { path: "/login", component: () => import('./components/Login.Vue') }
]

const router = VueRouter.createRouter({
    history: VueRouter.createWebHistory(),
    routes
})

const pinia = createPinia()

const app = createApp(App)
app.use(pinia)
app.use(router)
app.mount("#app")
