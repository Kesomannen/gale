<script lang="ts">
	import Label from '$lib/components/Label.svelte';
	import { invokeCommand } from '$lib/invoke';
	import { apiKeyPopupOpen } from './ApiKeyPopup.svelte';
	import Icon from '@iconify/svelte';
	import { Button } from 'bits-ui';

	let hasToken = false;

	$: {
		$apiKeyPopupOpen;
		refresh();
	}

	async function refresh() {
		hasToken = await invokeCommand('has_thunderstore_token')
	}
</script>

<div class="flex items-center">
	<Label text="Thunderstore API token">
		Thunderstore API token to use for modpack publishing. Once this is set, you will <b>not</b> be able
		to view the token again.
	</Label>

	<Button.Root
		class="flex flex-grow px-3 py-1 items-center text-right rounded-lg group bg-gray-900 truncate
                border border-gray-500 border-opacity-0 hover:border-opacity-100 relative"
		on:click={() => ($apiKeyPopupOpen = true)}
	>
		<div class="mr-2 rounded">
			<Icon
				icon={hasToken ? 'mdi:key' : 'mdi:key-remove'}
				class="align-middle text-slate-300 group-hover:text-slate-200"
			/>
		</div>

		<div class="text-slate-300 group-hover:text-slate-200 truncate">
			{hasToken ? 'Click to override token' : 'Not set'}
		</div>

		<slot name="field" />
	</Button.Root>
</div>
