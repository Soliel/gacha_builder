<script setup lang="ts">
import LoginWithDiscord from './components/LoginWithDiscord.vue';

</script>

<template>
  <header>
    <div class="flex w-full bg-zinc-700 border-b border-style-solid border-zinc-900 min-h-12 p-y-5">
      <LoginWithDiscord class="m-auto justify-self-end"></LoginWithDiscord>
    </div>
  </header>

  <main class="">
    <RouterView></RouterView>
  </main>
</template>
