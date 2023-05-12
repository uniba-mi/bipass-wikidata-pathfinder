where_block_template = Template("""
$subject_id $predicate_id $object_id .

FILTER ( CONTAINS( str($subject_id), "http://www.wikidata.org/entity/Q" ) ) .
FILTER ( CONTAINS( str($predicate_id), "http://www.wikidata.org/prop/direct/P" ) ) .
FILTER ( CONTAINS( str($object_id), "http://www.wikidata.org/entity/Q" ) ) .

$subject_id rdfs:label $the_subject_label .
FILTER ( lang($the_subject_label) = "en" ) .

$subject_id schema:description $the_subject_description .
FILTER ( lang($the_subject_description) = "en" ) .

$object_id rdfs:label $the_object_label .
FILTER ( lang($the_object_label) = "en" ) .
FILTER ( !CONTAINS ( $the_object_label, "Wiki" ) ) .
FILTER REGEX ( $the_object_label, "^[A-z0-9 -]+$$" ) .

$object_id schema:description $the_object_description .
FILTER ( lang($the_object_description) = "en" ) .
FILTER ( !CONTAINS ( $the_object_description, "Wiki" ) ) .

$foo wikibase:directClaim $predicate_id .
$foo rdfs:label $the_predicate_label.
FILTER ( lang($the_predicate_label) = "en" ) .
FILTER ( !CONTAINS ( $the_predicate_label, "Wiki" ) ) .
FILTER ( $predicate_id != wdt:P1343 ) .""")