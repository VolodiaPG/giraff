import enoslib as en

from emul.bpf import NetemBPF

_ = en.init_logging()

conf = (
    en.VMonG5kConf.from_settings( walltime="01:00:00", job_name="enos_bpf_netem", image="/home/volparolguarino/nixos.qcow2")
    .add_machine(
        roles=["city", "paris"],
        cluster="paravance",
                flavour="large"

    )
    .add_machine(
        roles=["city", "berlin"],
        cluster="paravance",
                flavour="large"

    )
    .add_machine(
        roles=["city", "londres"],
        cluster="paravance",
                flavour="large"

    )
    .finalize()
)
conf.finalize()

provider = en.VMonG5k(conf)
roles, networks = provider.init(force_deploy= True)
job = provider.g5k_provider.jobs[0]  # type: ignore

ips = [vm.address for vm in roles["city"]]

en.g5k_api_utils.enable_home_for_job(job, ips)

roles = en.sync_info(roles, networks)
netem = NetemBPF()

(
    netem.add_constraints(
        src=roles["paris"],
        dest=roles["londres"],
        delay=5,
        rate=1_000_000_000,
        symetric=True,
    )
    .add_constraints(
        src=roles["paris"],
        dest=roles["berlin"],
        delay=10,
        rate=1_000_000_000,
        symetric=True,
    )

    .add_constraints(
        src=roles["londres"],
        dest=roles["berlin"],
        delay=10,
        rate=1_000_000_000,
        symetric=True,
    )
)
netem.deploy()
netem.validate()

