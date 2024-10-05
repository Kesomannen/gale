<script lang="ts">
	import Icon from '@iconify/svelte';
	import { Button } from 'bits-ui';
	import Label from './Label.svelte';
	import { t } from '$i18n';

	export let label: string = '';
	export let onClick: () => void;
	export let value: string | null;
	export let icon: string = 'mdi:folder';

	$: hasValue = value && value.length > 0;
</script>

<div class="relative flex items-center">
	{#if label}
		<Label text={label}>
			<slot />
		</Label>
	{/if}

	<Button.Root
		class="group flex flex-grow basis-0 items-center truncate rounded-lg border border-gray-500 border-opacity-0 bg-gray-900 px-3 py-1 text-right hover:border-opacity-100"
		on:click={onClick}
	>
		<div class="mr-2 rounded">
			<Icon {icon} class="align-middle text-slate-300" />
		</div>

		<div class="truncate text-slate-300" style="direction: rtl;">
			&#x200E;
			{hasValue ? value : t('Not set')}
		</div>

		<slot name="field" />
	</Button.Root>
</div>
