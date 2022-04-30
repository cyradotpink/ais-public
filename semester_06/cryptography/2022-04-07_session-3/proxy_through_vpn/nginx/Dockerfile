FROM nginx:alpine

RUN apk add bash
RUN apk add openvpn

ARG vpn_user
ARG vpn_pass

COPY ./hshl.ovpn /hshl.ovpn
# COPY ./vpnauth.txt /vpnauth.txt
COPY ./nginx.conf /etc/nginx/templates/default.conf.template
COPY ./ovpn-entrypoint.sh /docker-entrypoint.d/ovpn.sh

RUN echo $vpn_user > /vpnauth.txt
RUN echo $vpn_pass >> /vpnauth.txt

#RUN mkdir /dev/net
#RUN mknod /dev/net/tun c 10 200
#RUN chmod a+rwx -R /dev/net
