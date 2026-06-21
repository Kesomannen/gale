<script lang="ts">
	import Icon from '@iconify/svelte';

	type Props = {
		value: string;
		placeholder: string;
		size?: 'sm' | 'base' | 'lg';
		onkeydown?: () => void;
		onclear?: () => void;
	};

	let { value = $bindable(), placeholder, size = 'base', onkeydown, onclear }: Props = $props();
</script>

<input
	type="text"
	class="bg-primary-900 valid:focus:ring-accent-500 text-primary-300 placeholder-primary-400 hover:ring-primary-500 w-full truncate rounded-lg px-10 py-1.5 invalid:ring-red-500 hover:ring-1 focus:ring-2 focus:outline-hidden text-{size}"
	bind:value
	{onkeydown}
	{placeholder}
/>
<Icon class="text-primary-400 absolute top-2 left-2.5 text-lg" icon="ph:magnifying-glass-fill" />

<button
	class="text-primary-400 hover:bg-primary-800 hover:text-primary-300 absolute top-1.25 right-1.5 rounded-md p-1 text-lg"
	onclick={() => {
		value = '';
		onclear?.();
	}}
>
	<Icon icon="ph:x-fill" />
</button>
