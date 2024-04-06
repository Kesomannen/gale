<script lang="ts">
	import Popup from '$lib/Popup.svelte';
	import { invokeCommand } from '$lib/invoke';
	import { refreshProfiles } from '$lib/profile';
	import { Button, Dialog } from 'bits-ui';

	export let open = false;

	let code: string;

	async function submit() {
		await invokeCommand('import_code', { key: code });
		refreshProfiles();
		open = false;
	}
</script>

<Popup title="Import from code" bind:open>
	<div class="flex">
		<input
			type="text"
			bind:value={code}
			on:keydown={(e) => {
				if (e.key !== 'Enter') return;
				submit();
			}}
			placeholder="Enter import code..."
			class="w-full my-2 px-3 py-2 rounded-lg bg-gray-900 text-slate-100 select-none"
		/>
		<Dialog.Close>
			<Button.Root
				class="rounded-xl px-4 py-2 ml-3 text-slate-100 bg-green-600 hover:bg-green-500 disabled:bg-gray-600/80 disabled:hover:bg-gray-600/80 disabled:text-gray-200/80"
				on:click={submit}
			>
				Submit
			</Button.Root>
		</Dialog.Close>
	</div>
</Popup>
