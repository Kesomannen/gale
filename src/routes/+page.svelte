<script lang="ts">
  let versions = [
    {
      num: '1.0.0',
      date: '2021-10-10',
      changes: [
        'Initial release'
      ]
    },
  ];
</script>

<div class="p-6 overflow-y-auto">
  <h1 class="text-white font-bold text-2xl">Welcome to Kesomannen Mod Manager!</h1>
  <h2 class="text-slate-100 font-semibold text-xl mt-2">Changelog</h2>
  {#each versions as version}
    <h3 class="text-slate-200 font-medium text-lg mt-2">Version {version.num}</h3>
    <ul>
      {#each version.changes as change}
        <li class="text-slate-300">- {change}</li>
      {/each}
    </ul>
  {/each}
</div>