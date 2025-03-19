<script lang="ts">
	import { Button } from 'bits-ui';

	export let disabled: boolean = false;
	export let color: 'accent' | 'primary' | 'red' = 'accent';
	export let fontWeight: 'normal' | 'medium' | 'semibold' = 'normal';
	export { className as class };

	let className = '';

	$: fontClass = {
		normal: 'font-normal',
		medium: 'font-medium',
		semibold: 'font-semibold'
	}[fontWeight];

	$: colorClass = {
		accent: 'enabled:hover:bg-accent-600 bg-accent-700',
		primary: 'enabled:hover:bg-primary-600 bg-primary-700',
		red: 'enabled:hover:bg-red-600 bg-red-700'
	}[color];
</script>

<Button.Root
	class="{className} {fontClass} {colorClass} disabled:opactiy-70 disabled:bg-primary-700 disabled:text-primary-400 inline-flex items-center overflow-hidden rounded-lg px-4 py-2 text-nowrap text-white disabled:cursor-not-allowed"
	on:click
	bind:disabled
>
	<slot />
</Button.Root>
