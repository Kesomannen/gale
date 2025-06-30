<script lang="ts">
	import { createBubbler } from 'svelte/legacy';

	const bubble = createBubbler();
	type Props = {
		value: string;
		mono?: boolean;
		[key: string]: any;
	};

	let { value = $bindable(), mono = false, ...rest }: Props = $props();
</script>

<textarea
	bind:value
	onchange={bubble('change')}
	{...rest}
	autocomplete="off"
	class="valid:focus:ring-accent-400 bg-primary-900 text-primary-300 placeholder-primary-400 hover:border-primary-500 h-40 min-h-11 w-full rounded-lg border border-transparent px-3 py-2 invalid:ring-red-500 focus:border-transparent focus:ring-2 focus:outline-hidden"
	class:font-mono={mono}
></textarea>
