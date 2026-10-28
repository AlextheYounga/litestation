<script setup lang="ts">
// Decorative welcome-screen backdrop. Purely presentational: a blueprint grid,
// a large database-cylinder line drawing, floating "table card" skeletons, and
// drifting SQL fragments layered to signal "database manager" at a glance.
// No props, no state; hidden from assistive technology.
</script>

<template>
  <div class="pointer-events-none absolute inset-0 -z-10 overflow-hidden" aria-hidden="true">
    <!-- Blueprint grid fading out from the upper left. -->
    <div class="backdrop-grid absolute inset-0"></div>

    <!-- Ambient glows. -->
    <div class="absolute -left-24 top-14 size-96 rounded-full bg-accent/7 blur-[110px]"></div>
    <div class="absolute -right-20 bottom-0 size-80 rounded-full bg-sky-400/5 blur-[110px]"></div>

    <!-- Database cylinder line art. -->
    <svg
      class="absolute -right-24 -top-10 hidden h-[32rem] w-[15rem] text-white/6 lg:block"
      viewBox="0 0 220 300"
      fill="none"
      stroke="currentColor"
      stroke-width="1.5"
    >
      <ellipse cx="110" cy="40" rx="92" ry="30" class="stroke-accent/30" />
      <path d="M18 40v210c0 16.6 41.3 30 92 30s92-13.4 92-30V40" />
      <path d="M18 110c0 16.6 41.3 30 92 30s92-13.4 92-30" />
      <path d="M18 180c0 16.6 41.3 30 92 30s92-13.4 92-30" />
    </svg>

    <!-- Floating table-card skeletons. -->
    <div class="backdrop-float absolute left-[3%] top-[10%] hidden w-36 rounded-lg border border-white/8 bg-white/[.03] p-3 sm:block">
      <div class="mb-2.5 h-2 w-14 rounded bg-accent/40"></div>
      <div class="space-y-1.5">
        <div class="h-1.5 rounded bg-white/10"></div>
        <div class="h-1.5 w-4/5 rounded bg-white/8"></div>
        <div class="h-1.5 w-3/5 rounded bg-white/8"></div>
      </div>
    </div>
    <div class="backdrop-float absolute bottom-[14%] left-[6%] hidden w-44 rounded-lg border border-white/8 bg-white/[.03] p-3 [animation-delay:-4s] lg:block">
      <div class="mb-2.5 h-2 w-16 rounded bg-accent/40"></div>
      <div class="space-y-1.5">
        <div class="h-1.5 rounded bg-white/10"></div>
        <div class="h-1.5 w-3/4 rounded bg-white/8"></div>
        <div class="h-1.5 w-4/5 rounded bg-white/8"></div>
        <div class="h-1.5 w-2/5 rounded bg-white/8"></div>
      </div>
    </div>
    <div class="backdrop-float absolute bottom-[26%] right-[4%] hidden w-40 rounded-lg border border-white/8 bg-white/[.03] p-3 [animation-delay:-7s] lg:block">
      <div class="mb-2.5 h-2 w-12 rounded bg-accent/40"></div>
      <div class="space-y-1.5">
        <div class="h-1.5 w-5/6 rounded bg-white/10"></div>
        <div class="h-1.5 w-2/3 rounded bg-white/8"></div>
        <div class="h-1.5 w-3/4 rounded bg-white/8"></div>
      </div>
    </div>

    <!-- Drifting SQL fragments. -->
    <p class="backdrop-drift absolute left-[8%] top-[6%] hidden font-mono text-2xs text-white/12 sm:block">SELECT name FROM sqlite_master;</p>
    <p class="backdrop-drift absolute bottom-[10%] right-[10%] hidden font-mono text-2xs text-white/10 [animation-delay:-5s] lg:block">CREATE TABLE users (id INTEGER PRIMARY KEY);</p>
  </div>
</template>