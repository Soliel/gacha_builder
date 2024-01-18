<script setup lang="ts">
import LoginWithDiscord from './components/LoginWithDiscord.vue';

</script>

<template>
  <header>
    <div class="flex justify-end w-full bg-zinc-700 shadow-zinc-950 shadow-md min-h-20">
      <LoginWithDiscord class=""></LoginWithDiscord>
    </div>
  </header>

  <main class="">
    <RouterView></RouterView>
  </main>
</template>
