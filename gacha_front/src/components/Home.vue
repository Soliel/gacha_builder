<!-- eslint-disable vue/multi-word-component-names -->
<script setup lang="ts">
import { onMounted, ref } from 'vue';
import { useUserStore } from '@/store';
import { useRouter } from 'vue-router';

const router = useRouter();
const userStore = useUserStore();
const userJson = ref();

onMounted(async () => {
    const resp = await fetch(`${location.origin}/auth/discord/me`)
    const body = await resp.json()
    userJson.value = body; 
})
</script>

<template>
    <p>{{userJson}}</p>
</template>