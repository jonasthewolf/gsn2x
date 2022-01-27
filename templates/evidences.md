
List of Evidences
{% set_global counter = 0 %}
{%- for id, node in nodes %} 
    {%- if id is starting_with("Sn") %}
        {%- set_global counter = counter + 1 %} 
{{ counter | ralign(width=evidences_width) }}. {{ id }}: {{ node["text"] }} 
{{ node["module"] | pad(width=evidences_width + 2) }}
        {%- if node["url"] %}
{{ node["url"] | pad(width=evidences_width + 2) }}
        {%- endif %} 
        {%- if layers %}
          {%- for layer in layers %}
            {%- if node[layer] %}
{{ layer | upper | pad(width=evidences_width + 2) }}: {{ node[layer] | trim }}
            {%- endif %}
          {%- endfor %}
        {%- endif %}
    {%- endif %} 
{%- endfor %} 
{%- if counter == 0 %}
No evidences found
{%- endif -%}