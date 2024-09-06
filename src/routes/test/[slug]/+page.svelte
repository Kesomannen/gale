<script lang="ts">
    import { page } from "$app/stores";
	import { invokeCommand } from "$lib/invoke";

    let id = $page.params.slug;

    let promise = invokeCommand<{
        name: string;
        owner: string;
        versions: {
            major: number;
            minor: number; 
            patch: number;
        }[]}>('plugin:gale-thunderstore|query_package', { id });
</script>

<div class="w-full max-w-screen-md mx-auto overflow-y-auto p-4">
	{#await promise}
		<p>loading...</p>
	{:then { name, owner, versions}}
		<div>
			<img
				src="https://gcdn.thunderstore.io/live/repository/icons/{owner}-{name}-{versions[0].major}.{versions[0].minor}.{versions[0].patch}.png"
				alt={name}
				class="size-32 rounded-lg"
			/>
			<h1 class="text-white font-bold text-2xl pt-3">{name}</h1>
		</div>
	{:catch error}
		<p>{error.message}</p>
	{/await}
</div>
