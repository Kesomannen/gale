<script lang="ts">
	import Dialog from '../ui/Dialog.svelte';
	import profiles from '$lib/state/profile.svelte';
	import Button from '../ui/Button.svelte';
	import MissingProfileItem from './MissingProfileItem.svelte';
	import type { MissingProfileAction } from '$lib/types';
	import { forgetProfile, setProfilePath } from '$lib/api/profile';
	import { pushInfoToast, pushToast } from '$lib/toast';

	const missingProfiles = $derived(profiles.list.filter((profile) => profile.missing));

	const actions: (MissingProfileAction | null)[] = $state(missingProfiles.map(() => null));

	async function submit() {
		let actionsToApply = missingProfiles.map((profile, i) => ({ profile, action: actions[i] }));
		let deleteCount = 0;
		let locateCount = 0;

		for (let { profile, action } of actionsToApply) {
			if (action === null) continue;

			if (action.type === 'delete') {
				await forgetProfile(profile.id);
				deleteCount++;
			} else if (action.type === 'locate') {
				await setProfilePath(profile.id, action.newPath);
				locateCount++;
			}
		}

		pushToast({
			type: 'info',
			name: 'Resolved missing profiles',
			message: `${locateCount} relocated, ${deleteCount} deleted.`
		});
	}
</script>

<Dialog open={missingProfiles.length > 0} canClose={false} title="Missing Profiles">
	<div class="text-primary-300 mb-2">
		<p class="mb-2">
			Gale detected missing profiles while loading. This can happen if you moved or deleted profiles
			from outside Gale.
		</p>
		<p>Please resolve each missing profile before continuing.</p>
	</div>

	<div class="border-primary-900 relative overflow-hidden rounded-lg border-2">
		{#each missingProfiles as profile, i (profile.id)}
			<MissingProfileItem
				{profile}
				value={actions[i]}
				onChanged={(action) => (actions[i] = action)}
			/>
		{/each}
	</div>

	<div class="mt-2 flex items-center justify-end">
		<Button color="primary" disabled={actions.some((action) => action === null)} onclick={submit}
			>Submit</Button
		>
	</div>
</Dialog>
