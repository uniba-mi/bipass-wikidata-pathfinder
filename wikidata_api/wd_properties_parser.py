import requests

html_doc = requests.get("https://www.wikidata.org/wiki/Wikidata:Database_reports/List_of_properties/all").text

from bs4 import BeautifulSoup
soup = BeautifulSoup(html_doc, "html.parser")

with open("wd_properties.csv", "w+") as file:

    line = "property_id,counts\n"
    file.write(line)

    result = []

    for tr in soup.find_all("tr"):

        tds = tr.find_all("td")

        if len(tds) < 5:
            continue

        data_type = tds[3].contents[0] if tds[3].contents else ""
        counts = tds[4].contents[0] if tds[4].contents else ""

        if data_type != "WI" or "M" not in counts:
            continue
        
        property_id = tds[0].a.contents[0] if tds[0].contents else ""

        result.append((property_id, int(counts.replace(' M', '').replace(',', ''))))

    result.sort(key=lambda tuple:tuple[1])

    for property_id, counts in result:
        line = f"{property_id},{counts}\n".replace("\n\n", "\n")
        file.write(line)
