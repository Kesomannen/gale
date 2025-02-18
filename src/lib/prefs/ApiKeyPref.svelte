<script lang="ts">
	import Info from '$lib/components/Info.svelte';
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
		hasToken = await invokeCommand('has_thunderstore_token');
	}
</script>

<div class="flex items-center">
	<Label>Thunderstore API token</Label>

	<Info>
		Thunderstore API token to use for modpack publishing. Once this is set, you will <b>not</b> be able
		to view the token again.
	</Info>

	<Button.Root
		class="group relative flex grow items-center truncate rounded-lg border border-transparent  bg-slate-900 px-3 py-1 text-right hover:border-slate-500"
		on:click={() => ($apiKeyPopupOpen = true)}
	>
		<div class="mr-2 rounded-sm">
			<Icon
				icon={hasToken ? 'mdi:key' : 'mdi:key-remove'}
				class="align-middle text-slate-300 group-hover:text-slate-200"
			/>
		</div>

		<div class="truncate text-slate-300 group-hover:text-slate-200">
			{hasToken ? 'Click to override token' : 'Not set'}
		</div>

		<slot name="field" />
	</Button.Root>
</div>
