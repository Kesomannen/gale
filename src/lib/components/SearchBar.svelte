<script lang="ts">
	import { createBubbler } from 'svelte/legacy';

	const bubble = createBubbler();
	import Icon from '@iconify/svelte';
	import { Button } from 'bits-ui';

	type Props = {
		value: string;
		placeholder: string;
		brightness?: number;
		size?: 'sm' | 'base' | 'lg';
	};

	let { value = $bindable(), placeholder, brightness = 900, size = 'base' }: Props = $props();
</script>

<input
	type="text"
	class="w-full rounded-lg px-10 py-1.5 bg-primary-{brightness} valid:focus:ring-accent-500 text-primary-300 placeholder-primary-400 hover:ring-primary-500 truncate invalid:ring-red-500 hover:ring-1 focus:ring-2 focus:outline-hidden text-{size}"
	bind:value
	onkeydown={bubble('keydown')}
	{placeholder}
/>
<Icon class="text-primary-400 absolute top-2 left-2.5 text-2xl" icon="mdi:magnify" />
<button
	class="text-primary-400 hover:bg-primary-800 hover:text-primary-300 absolute top-[0.4rem] right-1.5 rounded-md p-1 text-xl"
	onclick={() => (value = '')}
>
	<Icon icon="mdi:close" />
</button>
