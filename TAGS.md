# Tags used / constructing the Overpass API query
* Taginfo Database keeps records of the most common tags present in
 the OpenStreetMap database.
* Tags have to be actively selected to be put inside the query result of the Overpass API (whitelist)
* some tags have no special category, but are important for the heatmap the routing algorithm will use to determine "good" routes
* OSM has an ID for each node, way and realtion. So merging individual queries should be possible

## heatmapping: how to categorize good and bad spots on the map
* every tag (OSM map object) shall receive a rating based on the influence to the wellbeing of the bike rider
* depending on the object the influence is only immediate (a tree next to the road) or from distance (a nice mountain in the background)
* every object produces a concentric wave ripple, where wavetops (usually one) mark the point of most positive influence.
* negative rated objects (such as nearby motorways, big active industrial facilites, area of shopping centres, ...) have an inversed ripple with a negative peak, starting form zero
* after gathering the dataset each object will be analyzed once and
the corresponding ripples literally added (also in a mathematical sense) to the heatmap
* heatmap precicion should be only "good enough"

## Selection criterion of tags used for routing
### key=[highway](https://wiki.openstreetmap.org/wiki/Key:highway) - roads and paths
* not every road is traversable by bike, especially highways are
forbidden for non-motor vehicles <br>
=> do not include `highway=motorway` and `highway=motorway_link` in query <br>
&nbsp;&nbsp;&nbsp;&nbsp; do not include `highway` tags with [access](https://wiki.openstreetmap.org/wiki/Tag:bicycle%3Dno) set as `bicycle=no` 
&nbsp;&nbsp;&nbsp;&nbsp; whole table of access restrictions in Germany [here](https://wiki.openstreetmap.org/wiki/OSM_tags_for_routing/Access_restrictions#Germany)
* `highway=trunk` and `highway=trunk_link` are excluded by default, but could have cycleways. Also excluded, because driving near trunk roads is not beautiful at all :D
* steps are not (easily) passable by bike
=> do not include `highway=steps`

=> following ways with the `highway` tag are needed <br>

| key | value | discard if subtag is present |  
| --- | ----- | ---------------------------- |
| highway | primary | bicycle=no \| bicycle=use_sidepath \| <br> (motorroad=yes & cycleway=* not present) |
| highway | primary_link | bicycle=no \| bicycle=use_sidepath \| <br> (motorroad=yes & cycleway=* not present) |
| highway | secondary | bicycle=no \| bicycle=use_sidepath \| <br> (motorroad=yes & cycleway=* not present) |
| highway | secondary_link | bicycle=no \| bicycle=use_sidepath \| <br> (motorroad=yes & cycleway=* not present) |
| highway | tertiary | bicycle=no \| bicycle=use_sidepath \| <br> (motorroad=yes & cycleway=* not present) |
| highway | tertiary_link | bicycle=no \| bicycle=use_sidepath \| <br> (motorroad=yes & cycleway=* not present) |
| highway | unclassified | bicycle=no \| bicycle=use_sidepath \| <br> (access=private & bicycle!=yes) \| <br> (motorroad=yes & cycleway=* not present) |
| highway | residential | bicycle=no \| <br> (access=private & bicycle!=yes) |
| highway | living_street | bicycle=no \| <br> (access=private & bicycle!=yes) |
| highway | service | bicycle=no \| <br> (access=private & bicycle!=yes) |
| highway | path | bicycle=no \| <br> (access=private & bicycle!=yes) |
| highway | track | bicycle=no \| <br> (access=private & bicycle!=yes) \| tracktype=grade5 |
| highway | cycleway | / |
| highway | footway | bicycle=no |
| highway | pedestrian | bicycle=no |

* note: the above table just gives a rough orientation over the needed tags and is not an implementation detail!
* every Way with tags `highway=*` and `bicycle_road=yes` (Bike only street) and `cyclestreet=yes` (bike priorized street) 
* special priority for cycleways and `bicycle_road=yes`
* additionally exclude all `highway=*` with `smoothness=very_bad,horrible,very_horrible,impassable`, [see here](https://wiki.openstreetmap.org/wiki/Key:smoothness)
* exclude all `highway=*` with `surface=stepping_stones,gravel,rock,pebblestone,mud,sand,woodcips`[see here](https://wiki.openstreetmap.org/wiki/Key:surface)
* some road elements can also interesting, for example
beautiful historic bridges ([historic](https://wiki.openstreetmap.org/wiki/Historic)) tag.


### tags that positively influence the heatmap
* any [natural](https://wiki.openstreetmap.org/wiki/Key:natural) tag, depending on their general properties <br>
shall add more or less to the heatmap. <br><br>
Some natural objects are better to be seen form away (for a great sight) and some have an immediate influence on the felt route quality that is taken. <br>
For example a huge mountain has only a positive influence in itself, if viewed from far away. Its heat cycle will look more like a concentric circle with the middle flattened out. On the other hand a tree has only immediate impact on the pleasantness of the route. So its heatmap impact will look like a pylon, if the z axis is the heat. <br><br>
Rivers are abstractly covered by `natural=water`.

* some [tourism](https://wiki.openstreetmap.org/wiki/Key:tourism) tags, namely:

| key | value | discard if subtag is present |  
| --- | ----- | ---------------------------- |
| tourism | alpine_hut | / |
| tourism | artwork | / |
| tourism | attraction | / |
| tourism | picnic_site | / |
| tourism | viewpoint | / |

* some [man_made](https://wiki.openstreetmap.org/wiki/Key:man_made)
tags, namely:

| key | value | discard if subtag is present |  
| --- | ----- | ---------------------------- |
| man_made | adit | / |
| man_made | bridge | start_date > 1950 (or assign heat to old bridges only) |
| man_made | cairn | / |
| man_made | cellar_entrance | / |
| man_made | column | historic!=yes |
| man_made | cross | / |
| man_made | flagpole | / |
| man_made | lighthouse | / |
| man_made | mineshaft | / |
| man_made | obelisk | / |
| man_made | observatory | / |
| man_made | pier | / |
| man_made | water_tower | / |
| man_made | watermill | / |
| man_made | windmill | / |
| man_made | windpump | / |
| man_made | drinking_fountain | / |

* some [historic](https://wiki.openstreetmap.org/wiki/Key:historic) tags: 

| key | value | discard if subtag is present |  
| --- | ----- | ---------------------------- |
| historic | memorial | / |
| historic | archaeological_site | / |
| historic | wayside_cross | / |
| historic | ruins | / |
| historic | wayside_shrine | / |
| historic | monument | / |
| historic | building | / |
| historic | castle | / |
| historic | citywalls | / |
| historic | heritage | / |
| historic | manor | / |
| historic | church | / |
| historic | fort | / |
| historic | city_gate | / |
| historic | house | / |
| historic | hollow_way | / |
| historic | wreck | / |
| historic | cannon | / |
| historic | aircraft | / |
| historic | farm | / |
| historic | tower | / |
| historic | monastery | / |
| historic | bridge | / |
| historic | cemetery | / |
| historic | aqueduct | / |
| historic | locomotive | / |
| historic | ship | / |
| historic | tank | / |
| historic | railway_car | / |
| historic | vehicle | / |


## basic overpass turbo query
```graphql
/*
This query looks for nodes, ways and relations 
with the given key/value combination.
Choose your region and hit the Run button above!
*/
[out:json][timeout:25];
// gather results
(
  // query part for: “historic=fort”
  node["historic"="fort"]({{bbox}});
  way["historic"="fort"]({{bbox}});
  relation["historic"="fort"]({{bbox}});
);
// print results
out body;
>;
out skel qt;
```

## interesting spots without known tags
* street arts (tourism=artwork, artwork_type=graffiti)









