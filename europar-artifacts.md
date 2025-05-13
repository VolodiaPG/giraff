# Overview for submission 318

The following document showcase how to run our fog testbed as close a possible to
the experiments we've done. It does not require access to the Grid'5000 infrastructure.
The compromise is that it uses vagrant to run the same VMs that would emulate
fog nodes in our paper but cannot fully emulate the network conditions. Also, it
is rather small scale. Thus, this guide has been written to showcase our
testbed, but does not replace running the experiments on Grid'5000.

Repository: https://github.com/volodiapg/giraff
Branch: vagrant

## Getting started

Refer to README.MD, Getting started section.

## Step-by-step instructions

Refer to README.MD. Settings used for generating the results in the paper have
been tagged and can reproduced by changing the variables accordingly.

Execution time depends on the configured number of placement algorithms
compared. We have observed execution times of 30 minutes for the start of the
VMs and experiments to run on our 22 cores, 32 GB RAM development machine.
