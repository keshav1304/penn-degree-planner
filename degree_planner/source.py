from bs4 import BeautifulSoup
import requests
import json

# Load your HTML (replace this with your own HTML source or file)
html_content = requests.get("https://catalog.upenn.edu/attributes/")

soup = BeautifulSoup(html_content.content, "html.parser")

# Find all tags with class "bubblelink code"
list = soup.find_all("ul")
all_li_tags = soup.find_all('li')

# Create a single Python list of the text content
li_contents = []
for li in all_li_tags:
    li_contents.append(li.get_text(strip=True)) # strip=True removes leading/trailing whitespace

import re

# Find all occurrences of text inside brackets
all_attributes = []
for li in li_contents:
    matches = re.findall(r'\((.*?)\)', li)
    if len(matches) > 1:
        all_attributes.append(matches[1])
    elif len(matches) > 0:
        all_attributes.append(matches[0])
print(len(all_attributes))

# Optional: write to a JSON or text file
with open("codes.txt", "w", encoding="utf-8") as f:
    for attr in all_attributes[700:]:
        # Load your HTML (replace this with your own HTML source or file)
        url = f"https://catalog.upenn.edu/attributes/{attr.lower()}/"
        print(url)
        html_content = requests.get(url)

        soup = BeautifulSoup(html_content.content, "html.parser")

        # Find all tags with class "bubblelink code"
        tags = soup.find_all(class_="bubblelink code")

        # Extract the text content
        codes = [tag.get_text(strip=True).replace("\xa0", " ") for tag in tags]
        codes = json.dumps(codes)

        pred = f"map.insert(\"{attr.upper()}\".to_string(), vec!["
        end = "].into_iter().map(|s| s.to_string()).collect();\n"
        codes = pred + str(codes) + end
        f.write(codes)
