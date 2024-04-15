<script lang="ts">
	import SvelteMarkdown from "svelte-markdown";

  export let source: string;
</script>

<div class="markdown">
  <SvelteMarkdown {source} />
</div>

<style global lang="postcss">
  .markdown :global(*) {
    @apply text-slate-300;
  }

  .markdown :global(h1),
  .markdown :global(h2) {
    @apply pt-4 pb-1 mb-3 border-b border-slate-200;
  }

  .markdown :global(h1) {
    @apply text-white font-bold text-2xl;
  }

  .markdown :global(h2) {
    @apply text-slate-100 font-semibold text-xl;
  }

  .markdown :global(h3) {
    @apply text-slate-100 font-semibold text-lg pb-1 pt-2;
  }

  .markdown :global(h4) {
    @apply text-slate-100 font-medium text-lg pb-0.5 pt-1;
  }

  .markdown :global(a) {
    @apply text-green-400;
  }

  .markdown :global(a):hover {
    @apply text-green-300 underline;
  }

  .markdown :global(p) {
    @apply my-2
  }

  .markdown :global(li) {
    @apply my-1;
  }

  .markdown :global(ul li) {
    @apply list-disc ml-8 pl-0.5;
  }

  .markdown :global(ol li) {
    @apply list-decimal ml-8 pl-0.5;
  }

  .markdown :global(pre) {
    @apply bg-gray-900 text-slate-300 py-1 px-3 rounded-md overflow-x-auto;
  }

  .markdown :global(code) {
    @apply bg-gray-900 text-slate-300 rounded-md
  }

  .markdown :global(img) {
    @apply my-2;
  }
</style>