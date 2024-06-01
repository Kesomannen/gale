<script lang="ts">
	import Markdown from '$lib/components/Markdown.svelte';
	import type { PageData } from './$types';

	export let data: PageData;
</script>

<div class="absolute right-6 bottom-1 text-slate-400">
	Gale v{data.version}
</div>

<div class="px-6 py-2 overflow-y-auto text-slate-100 w-full">
	<Markdown source={data.changelog} />
</div>