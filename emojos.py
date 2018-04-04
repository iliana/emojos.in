# Copyright (c) 2018 iliana weller
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

import operator
import requests
import urllib.parse

from flask import Flask, redirect, render_template, request, url_for

app = Flask(__name__)


@app.route('/<domain>')
def emojo(domain):
    try:
        url = urllib.parse.urlunsplit(
            ('https', domain, '/api/v1/custom_emojis', '', ''))
        emojo = sorted(filter(lambda x: x['visible_in_picker'],
                              requests.get(url).json()),
                       key=operator.itemgetter('shortcode'))
        return render_template('emojo.html', domain=domain, emojo=emojo)
    except requests.exceptions.RequestException as e:
        return render_template('oh_no.html')


@app.route('/favicon.ico')
@app.route('/robots.txt')
def no_content():
    return ('', 204)


@app.route('/', methods=('GET', 'POST'))
def index():
    if request.method == 'POST':
        if 'instance' in request.form:
            return redirect(url_for('emojo', domain=request.form['instance']))
        else:
            return redirect(url_for('index'))
    else:
        return render_template('index.html')
