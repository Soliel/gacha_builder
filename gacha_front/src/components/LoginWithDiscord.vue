<script setup lang="ts">
import { useRouter } from 'vue-router';

const router = useRouter();

function login() {
    location.assign(`${location.origin}/auth/discord?returnto=${location.pathname}`)
}
</script>
<template>
    <button @click=login class="flex p-2 m-3 shadow-zinc-950 shadow-md justify-center align-middle items-center rounded-xl bg-lilac-400 hover:bg-lilac-500 active:bg-lilac-600 active:shadow-inner">
        <img class="h-7 mr-2 self-center" src="../assets/discord-mark-black.svg" />
        Login with Discord
    </button>
</template>