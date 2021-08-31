# Copyright (c) 2021 iliana etaoin
#
# This program is free software: you can redistribute it and/or modify it under
# the terms of the GNU Affero General Public License as published by the Free
# Software Foundation, either version 3 of the License, or (at your option) any
# later version.
#
# This program is distributed in the hope that it will be useful, but WITHOUT
# ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS
# FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more
# details.
#
# You should have received a copy of the GNU Affero General Public License
# along with this program.  If not, see <https://www.gnu.org/licenses/>.

import urllib.parse

import botocore.session
import requests
import serverless_wsgi
from collections import defaultdict
from dataclasses import dataclass
from flask import Flask, redirect, render_template, request, url_for

app = Flask(__name__)


def slug_filter(s):
    return s.lower().replace(" ", "-")


app.jinja_env.filters["slug"] = slug_filter


@dataclass
class Emoj:
    __slots__ = frozenset({"shortcode", "url"})
    shortcode: str
    url: str


@app.route("/<domain>")
def emojo(domain):
    if request.args.get("show_all", "") == "on":
        show_all = True
    else:
        show_all = False
    if request.args.get("show_animated", "") == "on":
        show_animated = True
    else:
        show_animated = False

    url = urllib.parse.urlunsplit(("https", domain, "/api/v1/custom_emojis", "", ""))
    try:
        response = requests.get(url)
    except requests.exceptions.RequestException:
        return render_template("oh_no.html", domain=domain)

    if response.status_code == 401:
        return render_template("forbidden.html", domain=domain)

    categories = defaultdict(list)
    for emoji in sorted(
        response.json(),
        # sort by category,
        # then name within each category,
        # then disambiguate by capitalization
        key=lambda x: (x.get("category", ""), x["shortcode"].lower(), x["shortcode"]),
    ):
        if not show_all and not emoji.get("visible_in_picker", True):
            continue

        url = emoji["url" if show_animated else "static_url"]
        categories[emoji.get("category")].append(
            Emoj(shortcode=emoji["shortcode"], url=url)
        )

    return render_template(
        "emojo.html", domain=domain, categories=categories, show_animated=show_animated
    )


@app.route("/favicon.ico")
@app.route("/robots.txt")
def no_content():
    return ("", 204)


@app.route("/code")
def code():
    context = request.environ.get("context")
    session = botocore.session.get_session()
    # region name is detected from lambda environment
    client = session.create_client("lambda")
    code = client.get_function(
        FunctionName=context.function_name, Qualifier=context.function_version
    )
    return redirect(code["Code"]["Location"], code=303)


@app.route("/", methods=("GET", "POST"))
def index():
    if request.method == "POST":
        if "instance" in request.form:
            show_all = request.form.get("show_all")
            show_animated = request.form.get("show_animated")
            return redirect(
                url_for(
                    "emojo",
                    domain=request.form["instance"],
                    show_all=show_all,
                    show_animated=show_animated,
                )
            )
        else:
            return redirect(url_for("index"))
    else:
        return render_template("index.html")


def handle_request(event, context):
    return serverless_wsgi.handle_request(app, event, context)
