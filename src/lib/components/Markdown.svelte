<script lang="ts">
	import SvelteMarkdown from "svelte-markdown";

  export let source: string;
</script>

<div class="markdown overflow-x-hidden">
  <SvelteMarkdown {source} />
</div>

<style global lang="postcss">
  .markdown :global(*) {
    @apply text-slate-300;
  }

  .markdown :global(h1),
  .markdown :global(h2) {
    @apply pt-4 pb-1 mb-3 border-b-2 border-slate-500;
  }

  .markdown :global(h1) {
    @apply text-white font-bold text-2xl;
  }

  .markdown :global(h2) {
    @apply text-slate-100 font-semibold text-xl;
  }

  .markdown :global(h3) {
    @apply text-slate-200 font-semibold text-lg pb-1 pt-2;
  }

  .markdown :global(h4) {
    @apply text-slate-300 font-medium text-lg pb-0.5 pt-1;
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

  .markdown :global(table) {
    @apply border border-gray-900 border-collapse max-w-full;
  }

  .markdown :global(th) {
    @apply bg-gray-900 text-slate-300 font-semibold px-2 py-1 text-left;
  }

  .markdown :global(tr) {
    @apply border-b border-gray-900;
  }

  .markdown :global(tr:nth-child(2n)) {
    @apply bg-gray-900;
  }

  .markdown :global(td) {
    @apply px-2 py-1 text-left;
  }
</style>