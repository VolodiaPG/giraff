# To call python from R
library(reticulate)
library(tidyverse)
library(igraph)
library(r2r)
library(formattable)

use_virtualenv('/home/volodia/Documents/faas_fog/enos/live/.venv/', required = TRUE)
import_from_path("monitoring", path = "/home/volodia/Documents/faas_fog/enos/live")
source_python('/home/volodia/Documents/faas_fog/enos/live/integration.py')

raw <- py$ADJACENCY

adjancy_transform <- function(x){
  ret = matrix(0,nrow=length(x),ncol=length(x))
  rownames(ret) <- c(sort(names(x)))
  colnames(ret) <- c(sort(names(x)))
  for (ii in seq_along(x)){
    for(jj in seq_along(x[[ii]])){
      ret[names(x)[[ii]],x[[ii]][[jj]][[1]]] <- strtoi(x[[ii]][[jj]][[2]])
    }
    
  }
  return(ret)
}

graph_matrix <- adjancy_transform(raw)
net <- graph_from_adjacency_matrix(as.matrix(graph_matrix),weighted = TRUE)
plot(net)

# distribution des fonctions
# politique basique edge first ou quoi

# ceb <- cluster_edge_betweenness(g) 
# 
# dendPlot(ceb, mode="hclust")
# plot(ceb, g) 


names_raw <- read.csv2(file = '/home/volodia/Documents/faas_fog/enos/live/metrics/names.csv', header = TRUE, sep = '\t')
names <- hashmap()
names[names_raw$instance] <- names_raw$name

# temps de deploiement vs nombre de noeuds dans le Fog
# temps de deploiement vs nombre de fonctions
# montrer 

# monitorer la mémoire et le cpu au niveau du noeud fog

ram_usage <- read.csv2(file = '/home/volodia/Documents/faas_fog/enos/live/metrics/fog_node_memory_used.csv', header = TRUE, sep = '\t')
ram_usage2 <- ram_usage %>%
  mutate(instance = names[instance]) %>%
  mutate(instance = as.character(instance)) %>%
  select(c(instance, timestamp, value)) %>%
  mutate(timestamp = as.numeric(as.character(timestamp)) ) %>%
  mutate(timestamp = as.POSIXct(timestamp, origin = "1970-01-01")) %>%
  mutate(value = as.numeric(as.character(value))) 

ggplot(ram_usage2,aes(x = timestamp, y= value, color=instance)) +
  geom_point() +
  geom_smooth()
  


bids_raw <- read.csv2(file = '/home/volodia/Documents/faas_fog/enos/live/metrics/fog_node_bids.csv', header = TRUE, sep = '\t')
bids_raw <- bids_raw %>%
  mutate(instance = names[instance]) %>%
  mutate(instance = as.character(instance)) %>%
  mutate(timestamp = as.numeric(as.character(timestamp)) ) %>%
  mutate(timestamp = as.POSIXct(timestamp, origin = "1970-01-01")) %>%
  mutate(value = as.numeric(as.character(value)) ) 

bids <- bids_raw %>%
  distinct(bid_id, value, .keep_all = TRUE) %>%
  select(c(instance, function_name, timestamp, value))

bids %>%
  ggplot(aes(x=function_name, y=value, color=instance)) +
  geom_violin()
  #geom_jitter(size=0.4, alpha=0.9)
  # scale_y_continuous(labels = scales::percent)


mem_raw <- read.csv2(file = '/home/volodia/Documents/faas_fog/enos/live/metrics/fog_node_memory_available.csv', header = TRUE, sep = '\t')
mem <- mem_raw %>%
  mutate(instance = names[instance]) %>%
  mutate(instance = as.character(instance)) %>%
  distinct(instance, value, .keep_all = TRUE) %>%
  select(c(instance, value)) %>%
  arrange(instance) %>%
  # group_by(instance) %>%
  mutate(dyn = formattable::percent(value / max(value))) %>% 
  mutate(value = as.numeric(as.character(value)) )

cpu_raw <- read.csv2(file = '/home/volodia/Documents/faas_fog/enos/live/metrics/fog_node_cpu_available.csv', header = TRUE, sep = '\t')
cpu <- cpu_raw %>%
  mutate(instance = names[instance]) %>%
  mutate(instance = as.character(instance)) %>%
  distinct(instance, value, .keep_all = TRUE) %>%
  select(c(instance, value)) %>%
  arrange(instance) %>%
  group_by(instance) %>%
  mutate(value = as.numeric(as.character(value)) ) 

PercentageColour <- function(x){colorRampPalette(c('white','purple'))(101)[round(x*100)+1]}

# size is ram compared to largest of the network
net_prez <- graph_from_adjacency_matrix(as.matrix(graph_matrix),weighted = TRUE)
V(net_prez)$size <- cpu$value * 4
V(net_prez)$color <- PercentageColour(mem$dyn)
E(net_prez)$width <- E(net_prez)$weight / 50
plot(net_prez, layout=layout_as_tree, edge.label = E(net_prez)$weight)

bids_won_raw <- bids_raw %>%
  select(c(instance, function_name, value))
  
bids_won_all_zero <- data.frame(instance=names_raw$name, n=0)
bids_won <- bids_won_raw %>%
  group_by(function_name) %>%
  slice(which.min(value)) %>%
  group_by(instance) %>%
  summarise(n = n()) %>%
  merge(bids_won, bids_won_all_zero, all = TRUE) %>% 
  group_by(instance) %>%
  summarise(across(everything(), sum)) %>%
  arrange

net_won <- graph_from_adjacency_matrix(as.matrix(graph_matrix),weighted = TRUE)
V(net_won)$size <- bids_won$n * 2
plot(net_won, layout = layout_as_tree)


mem_used_raw <- read.csv2(file = '/home/volodia/Documents/faas_fog/enos/live/metrics/fog_node_memory_used.csv', header = TRUE, sep = '\t')
mem_used <- mem_used_raw %>%
  mutate(instance = names[instance]) %>%
  mutate(instance = as.character(instance)) %>%
  select(c(instance, timestamp, value)) %>%
  mutate(timestamp = as.numeric(as.character(timestamp)) ) %>%
  mutate(timestamp = as.POSIXct(timestamp, origin = "1970-01-01")) %>%
  mutate(value = as.numeric(as.character(value)) ) 

mem_used %>%
  ggplot(aes(x=timestamp, y=value, color=instance)) +
  geom_point() 

latency_raw <- read.csv2(file = '/home/volodia/Documents/faas_fog/enos/live/metrics/fog_node_neighbors_latency.csv', header = TRUE, sep = '\t')
latency <- latency_raw %>%
  mutate(instance = names[instance]) %>%
  mutate(instance = as.character(instance)) %>%
  mutate(instance_to = names[instance_to]) %>%
  mutate(instance_to = as.character(instance_to)) %>%
  mutate(timestamp = as.numeric(as.character(timestamp)) ) %>%
  mutate(timestamp = as.POSIXct(timestamp, origin = "1970-01-01")) %>%
  mutate(value = as.numeric(as.character(value)))

latency2 <- latency %>%
  group_by(instance, instance_to) %>%
  summarise(mean = mean(value))

bids_won_function <- bids_won_raw %>%
  group_by(function_name) %>%
  slice(which.min(value)) %>%
  mutate(winner = instance)

bids_function <- bids_raw %>%
  select(c(instance, function_name, value)) %>%
  distinct() %>%
  inner_join(bids_won_function, by=c("function_name"))


bids_function %>%
  ggplot(aes(x=function_name, y=value.x, group=function_name, color=winner, label=instance.x)) +
  geom_boxplot() +
  geom_point() +
  geom_text(hjust=0, vjust=0) 


