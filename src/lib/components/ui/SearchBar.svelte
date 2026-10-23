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
	class="valid:focus:ring-accent-600 text-primary-700 placeholder-primary-500 hover:ring-primary-400 bg-primary-100 w-full truncate rounded-lg px-10 py-1.5 invalid:ring-red-500 hover:ring-1 focus:ring-2 focus:outline-hidden text-{size} dark:bg-primary-900 dark:valid:focus:ring-accent-500 dark:text-primary-300 dark:placeholder-primary-400 dark:hover:ring-primary-500"
	bind:value
	{onkeydown}
	{placeholder}
/>
<Icon
	class="text-primary-500 dark:text-primary-400 absolute top-1.75 left-2.5 text-2xl"
	icon="mdi:magnify"
/>

<button
	class="text-primary-500 hover:text-primary-700 dark:text-primary-400 dark:hover:bg-primary-800 dark:hover:text-primary-300 hover:bg-primary-200 absolute top-1.25 right-1.5 rounded-md p-1 text-xl"
	onclick={() => {
		value = '';
		onclear?.();
	}}
>
	<Icon icon="mdi:close" />
</button>
